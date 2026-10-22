//! The user's data outside projects: device profiles
//! (`docs/spec/devices.md`), material libraries (`docs/spec/materials.md`)
//! and test presets (`docs/spec/test-patterns.md`), with their files in the
//! configuration folder.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod camera;
pub mod library;
pub mod presets;
pub mod profile;
pub mod starter;

pub use camera::{Camera, Source};
pub use library::{Key, Library, Material};
pub use presets::Preset;
pub use profile::DeviceProfile;

/// Something about user data went wrong. The message is ready to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkshopError(pub String);

impl WorkshopError {
    /// An error with this message.
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for WorkshopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for WorkshopError {}

const PROFILES_FILE: &str = "devices.json";
const PROFILES_FORMAT: &str = "lasercraft-devices";
const PROFILE_FORMAT: &str = "lasercraft-device";
const PRESETS_FILE: &str = "test-presets.json";
const STATE_FILE: &str = "workshop.json";
const STATE_FORMAT: &str = "lasercraft-workshop";

/// The configuration folder (`docs/spec/devices.md`), if one can be found.
#[must_use]
pub fn config_dir() -> Option<PathBuf> {
    let var = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    if let Some(dir) = var("LASERCRAFT_CONFIG_DIR") {
        return Some(dir);
    }
    if cfg!(target_os = "windows") {
        return var("APPDATA").map(|d| d.join("LaserCraft"));
    }
    if cfg!(target_os = "macos") {
        return var("HOME").map(|h| h.join("Library/Application Support/LaserCraft"));
    }
    var("XDG_CONFIG_HOME")
        .map(|d| d.join("lasercraft"))
        .or_else(|| var("HOME").map(|h| h.join(".config/lasercraft")))
}

/// Writes `text` next to `path` and renames it over, so a failed write
/// never leaves half a file.
fn write_atomic(path: &Path, text: &str) -> Result<(), WorkshopError> {
    let fail =
        |e: std::io::Error| WorkshopError::new(format!("cannot write {}: {e}", path.display()));
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(fail)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, text).map_err(fail)?;
    fs::rename(&tmp, path).map_err(fail)
}

fn read(path: &Path) -> Result<String, WorkshopError> {
    fs::read_to_string(path)
        .map_err(|e| WorkshopError::new(format!("cannot open {}: {e}", path.display())))
}

fn pretty<T: Serialize>(value: &T) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_default();
    text.push('\n');
    text
}

fn same(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

/// An open material library.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenLibrary {
    /// The library.
    pub library: Library,
    /// Its file, once opened from or saved to one.
    pub path: Option<PathBuf>,
    /// Changed since it was opened or saved.
    pub modified: bool,
    /// The read-only starter library.
    pub built_in: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StateFile {
    #[serde(default)]
    format: String,
    #[serde(default)]
    version: u32,
    #[serde(default)]
    libraries: Vec<PathBuf>,
    #[serde(default)]
    active: Option<String>,
}

/// The user's device profiles, open libraries and test presets.
#[derive(Debug, Clone, PartialEq)]
pub struct Workshop {
    dir: Option<PathBuf>,
    profiles: Vec<DeviceProfile>,
    default: Option<String>,
    libraries: Vec<OpenLibrary>,
    active: usize,
    presets: Vec<Preset>,
    last: serde_json::Map<String, Value>,
    /// A write failure from the last change, to report.
    warning: Option<String>,
}

impl Default for Workshop {
    fn default() -> Self {
        Self::in_memory()
    }
}

impl Workshop {
    /// User data kept in memory only (the CLI without `--config`, tests):
    /// no profiles, the starter library open and active.
    #[must_use]
    pub fn in_memory() -> Self {
        Self {
            dir: None,
            profiles: Vec::new(),
            default: None,
            libraries: vec![OpenLibrary {
                library: starter::library(),
                path: None,
                modified: false,
                built_in: true,
            }],
            active: 0,
            presets: Vec::new(),
            last: serde_json::Map::new(),
            warning: None,
        }
    }

    /// Reads the user data in `dir`, creating nothing until something
    /// changes. Returns the workshop and messages about files that could
    /// not be read (their data is left out).
    #[must_use]
    pub fn load(dir: &Path) -> (Self, Vec<String>) {
        let mut ws = Self {
            dir: Some(dir.to_owned()),
            ..Self::in_memory()
        };
        let mut messages = Vec::new();
        let profiles = dir.join(PROFILES_FILE);
        if profiles.exists() {
            match read(&profiles).and_then(|t| parse_profiles(&t)) {
                Ok(file) => {
                    ws.profiles = file.profiles;
                    ws.default = file.default.filter(|d| ws.profile(d).is_some());
                }
                Err(e) => messages.push(format!("Device profiles were not loaded: {e}")),
            }
        }
        let presets = dir.join(PRESETS_FILE);
        if presets.exists() {
            match read(&presets).and_then(|t| parse_presets(&t)) {
                Ok(file) => {
                    ws.presets = file.presets;
                    ws.last = file.last;
                }
                Err(e) => messages.push(format!("Test presets were not loaded: {e}")),
            }
        }
        let state = dir.join(STATE_FILE);
        if let Ok(text) = fs::read_to_string(&state)
            && let Ok(state) = serde_json::from_str::<StateFile>(&text)
        {
            for path in state.libraries {
                match read(&path).and_then(|t| Library::from_json(&t)) {
                    Ok(library) if ws.library_index(&library.name).is_none() => {
                        ws.libraries.push(OpenLibrary {
                            library,
                            path: Some(path),
                            modified: false,
                            built_in: false,
                        });
                    }
                    Ok(_) => {}
                    Err(e) => messages.push(format!("A material library was not reopened: {e}")),
                }
            }
            if let Some(i) = state.active.and_then(|n| ws.library_index(&n)) {
                ws.active = i;
            }
        }
        (ws, messages)
    }

    /// The configuration folder, if the workshop has one.
    #[must_use]
    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    /// Takes the message about a failed write, if the last change could
    /// not be saved.
    pub fn take_warning(&mut self) -> Option<String> {
        self.warning.take()
    }

    fn persist(&mut self, file: &str, text: &str) {
        if let Some(dir) = &self.dir
            && let Err(e) = write_atomic(&dir.join(file), text)
        {
            self.warning = Some(e.to_string());
        }
    }

    fn save_profiles(&mut self) {
        let text = pretty(&profile::ProfilesFile {
            format: PROFILES_FORMAT.to_owned(),
            version: 1,
            default: self.default.clone(),
            profiles: self.profiles.clone(),
        });
        self.persist(PROFILES_FILE, &text);
    }

    fn save_presets(&mut self) {
        let text = pretty(&presets::PresetsFile {
            format: presets::FORMAT.to_owned(),
            version: 1,
            presets: self.presets.clone(),
            last: self.last.clone(),
        });
        self.persist(PRESETS_FILE, &text);
    }

    fn save_state(&mut self) {
        let text = pretty(&StateFile {
            format: STATE_FORMAT.to_owned(),
            version: 1,
            libraries: self
                .libraries
                .iter()
                .filter_map(|l| l.path.clone())
                .collect(),
            active: Some(self.libraries[self.active].library.name.clone()),
        });
        self.persist(STATE_FILE, &text);
    }

    // ---- Device profiles ----

    /// Every profile, in the order they were added.
    #[must_use]
    pub fn profiles(&self) -> &[DeviceProfile] {
        &self.profiles
    }

    /// The profile with this name (case-insensitive).
    #[must_use]
    pub fn profile(&self, name: &str) -> Option<&DeviceProfile> {
        self.profiles.iter().find(|p| same(&p.name, name))
    }

    fn profile_index(&self, name: &str) -> Result<usize, WorkshopError> {
        self.profiles
            .iter()
            .position(|p| same(&p.name, name))
            .ok_or_else(|| {
                WorkshopError::new(format!("no device profile named \"{}\"", name.trim()))
            })
    }

    /// The default profile's name.
    #[must_use]
    pub fn default_profile(&self) -> Option<&str> {
        self.default.as_deref()
    }

    fn free(&self, name: &str) -> Result<(), WorkshopError> {
        if self.profile(name).is_some() {
            Err(WorkshopError::new(format!(
                "a device profile named \"{}\" already exists",
                name.trim()
            )))
        } else {
            Ok(())
        }
    }

    /// Adds a profile; the first one becomes the default.
    ///
    /// # Errors
    ///
    /// The name is taken or a field is invalid.
    pub fn add_profile(&mut self, profile: DeviceProfile) -> Result<(), WorkshopError> {
        profile.validate()?;
        self.free(&profile.name)?;
        if self.profiles.is_empty() {
            self.default = Some(profile.name.clone());
        }
        self.profiles.push(profile);
        self.save_profiles();
        Ok(())
    }

    /// Changes one field of a profile.
    ///
    /// # Errors
    ///
    /// No such profile, an unknown key or an invalid value.
    pub fn set_profile(&mut self, name: &str, key: &str, value: &str) -> Result<(), WorkshopError> {
        let i = self.profile_index(name)?;
        self.profiles[i].set(key, value)?;
        self.save_profiles();
        Ok(())
    }

    /// Renames a profile, keeping the default mark.
    ///
    /// # Errors
    ///
    /// No such profile, or the new name is invalid or taken.
    pub fn rename_profile(&mut self, old: &str, new: &str) -> Result<(), WorkshopError> {
        let i = self.profile_index(old)?;
        let new = profile::check_name(new)?;
        if !same(old, &new) {
            self.free(&new)?;
        }
        if self.default.as_deref().is_some_and(|d| same(d, old)) {
            self.default = Some(new.clone());
        }
        self.profiles[i].name = new;
        self.save_profiles();
        Ok(())
    }

    /// Copies a profile under a new name.
    ///
    /// # Errors
    ///
    /// No such profile, or the new name is invalid or taken.
    pub fn duplicate_profile(&mut self, name: &str, new: &str) -> Result<(), WorkshopError> {
        let mut copy = self.profiles[self.profile_index(name)?].clone();
        copy.name = profile::check_name(new)?;
        self.add_profile(copy)
    }

    /// Removes a profile.
    ///
    /// # Errors
    ///
    /// No such profile.
    pub fn delete_profile(&mut self, name: &str) -> Result<DeviceProfile, WorkshopError> {
        let i = self.profile_index(name)?;
        if self.default.as_deref().is_some_and(|d| same(d, name)) {
            self.default = None;
        }
        let removed = self.profiles.remove(i);
        self.save_profiles();
        Ok(removed)
    }

    /// Sets or clears the default profile.
    ///
    /// # Errors
    ///
    /// No such profile.
    pub fn set_default(&mut self, name: Option<&str>) -> Result<(), WorkshopError> {
        self.default = match name {
            Some(n) => Some(self.profiles[self.profile_index(n)?].name.clone()),
            None => None,
        };
        self.save_profiles();
        Ok(())
    }

    /// Adds the profile in a `.lcdevice` file; a clashing name gets " (2)",
    /// " (3)", …. Returns the name it was added under.
    ///
    /// # Errors
    ///
    /// The file cannot be read or is not a valid profile file.
    pub fn import_profile(&mut self, path: &Path) -> Result<String, WorkshopError> {
        let file: profile::ProfileFile = serde_json::from_str(&read(path)?)
            .map_err(|e| WorkshopError::new(format!("not a LaserCraft device profile: {e}")))?;
        if file.format != PROFILE_FORMAT {
            return Err(WorkshopError::new("not a LaserCraft device profile"));
        }
        if file.version > 1 {
            return Err(WorkshopError::new(
                "this device profile was made by a newer LaserCraft; update LaserCraft to import it",
            ));
        }
        let mut profile = file.profile;
        let base = profile.name.clone();
        let mut n = 2;
        while self.profile(&profile.name).is_some() {
            profile.name = format!("{base} ({n})");
            n += 1;
        }
        let name = profile.name.clone();
        self.add_profile(profile)?;
        Ok(name)
    }

    /// Writes one profile as a `.lcdevice` file.
    ///
    /// # Errors
    ///
    /// No such profile, or the file cannot be written.
    pub fn export_profile(&self, name: &str, path: &Path) -> Result<(), WorkshopError> {
        let profile = self.profiles[self.profile_index(name)?].clone();
        write_atomic(
            path,
            &pretty(&profile::ProfileFile {
                format: PROFILE_FORMAT.to_owned(),
                version: 1,
                profile,
            }),
        )
    }

    // ---- Cameras (M9) ----

    /// The cameras of a profile.
    ///
    /// # Errors
    ///
    /// No such profile.
    pub fn cameras(&self, profile: &str) -> Result<&[Camera], WorkshopError> {
        Ok(&self.profiles[self.profile_index(profile)?].cameras)
    }

    /// One camera of a profile.
    ///
    /// # Errors
    ///
    /// No such profile or camera.
    pub fn camera(&self, profile: &str, name: &str) -> Result<&Camera, WorkshopError> {
        self.cameras(profile)?
            .iter()
            .find(|c| same(&c.name, name))
            .ok_or_else(|| WorkshopError::new(format!("no camera named \"{}\"", name.trim())))
    }

    /// Changes one camera of a profile and saves; the camera is unchanged
    /// when `change` fails.
    ///
    /// # Errors
    ///
    /// No such profile or camera, or what `change` returns.
    pub fn edit_camera<T>(
        &mut self,
        profile: &str,
        name: &str,
        change: impl FnOnce(&mut Camera) -> Result<T, WorkshopError>,
    ) -> Result<T, WorkshopError> {
        let p = self.profile_index(profile)?;
        let c = self.profiles[p]
            .cameras
            .iter()
            .position(|c| same(&c.name, name))
            .ok_or_else(|| WorkshopError::new(format!("no camera named \"{}\"", name.trim())))?;
        let mut copy = self.profiles[p].cameras[c].clone();
        let out = change(&mut copy)?;
        copy.validate()?;
        self.profiles[p].cameras[c] = copy;
        self.save_profiles();
        Ok(out)
    }

    /// Adds a camera to a profile.
    ///
    /// # Errors
    ///
    /// No such profile, the name is taken, or the camera is invalid.
    pub fn add_camera(&mut self, profile: &str, camera: Camera) -> Result<(), WorkshopError> {
        camera.validate()?;
        let p = self.profile_index(profile)?;
        if self.profiles[p]
            .cameras
            .iter()
            .any(|c| same(&c.name, &camera.name))
        {
            return Err(WorkshopError::new(format!(
                "a camera named \"{}\" already exists",
                camera.name
            )));
        }
        self.profiles[p].cameras.push(camera);
        self.save_profiles();
        Ok(())
    }

    /// Removes a camera with its correction and alignments.
    ///
    /// # Errors
    ///
    /// No such profile or camera.
    pub fn remove_camera(&mut self, profile: &str, name: &str) -> Result<Camera, WorkshopError> {
        let p = self.profile_index(profile)?;
        let c = self.profiles[p]
            .cameras
            .iter()
            .position(|c| same(&c.name, name))
            .ok_or_else(|| WorkshopError::new(format!("no camera named \"{}\"", name.trim())))?;
        let removed = self.profiles[p].cameras.remove(c);
        self.save_profiles();
        Ok(removed)
    }

    /// Renames a camera.
    ///
    /// # Errors
    ///
    /// No such camera, or the new name is invalid or taken.
    pub fn rename_camera(
        &mut self,
        profile: &str,
        old: &str,
        new: &str,
    ) -> Result<(), WorkshopError> {
        let new = profile::check_name(new)?;
        if !same(old, &new) && self.camera(profile, &new).is_ok() {
            return Err(WorkshopError::new(format!(
                "a camera named \"{new}\" already exists"
            )));
        }
        self.edit_camera(profile, old, |c| {
            c.name = new;
            Ok(())
        })
    }

    /// Adds the camera in a `.lccam` file to a profile; a clashing name
    /// gets " (2)", " (3)", …. Returns the name it was added under.
    ///
    /// # Errors
    ///
    /// No such profile, or the file is not a valid camera file.
    pub fn import_camera(&mut self, profile: &str, path: &Path) -> Result<String, WorkshopError> {
        let file: camera::CameraFile = serde_json::from_str(&read(path)?)
            .map_err(|e| WorkshopError::new(format!("not a LaserCraft camera file: {e}")))?;
        if file.format != camera::FORMAT {
            return Err(WorkshopError::new("not a LaserCraft camera file"));
        }
        if file.version > 1 {
            return Err(WorkshopError::new(
                "this camera file was made by a newer LaserCraft; update LaserCraft to import it",
            ));
        }
        let mut cam = file.camera;
        let base = cam.name.clone();
        let mut n = 2;
        while self.camera(profile, &cam.name).is_ok() {
            cam.name = format!("{base} ({n})");
            n += 1;
        }
        let name = cam.name.clone();
        self.add_camera(profile, cam)?;
        Ok(name)
    }

    /// Writes one camera as a `.lccam` file.
    ///
    /// # Errors
    ///
    /// No such camera, or the file cannot be written.
    pub fn export_camera(
        &self,
        profile: &str,
        name: &str,
        path: &Path,
    ) -> Result<(), WorkshopError> {
        let cam = self.camera(profile, name)?.clone();
        write_atomic(
            path,
            &pretty(&camera::CameraFile {
                format: camera::FORMAT.to_owned(),
                version: 1,
                camera: cam,
            }),
        )
    }

    // ---- Material libraries ----

    /// Every open library; the starter library is first.
    #[must_use]
    pub fn libraries(&self) -> &[OpenLibrary] {
        &self.libraries
    }

    /// The active library.
    #[must_use]
    pub fn active(&self) -> &OpenLibrary {
        &self.libraries[self.active]
    }

    fn library_index(&self, name: &str) -> Option<usize> {
        self.libraries
            .iter()
            .position(|l| same(&l.library.name, name))
    }

    /// The open library with this name.
    #[must_use]
    pub fn library(&self, name: &str) -> Option<&OpenLibrary> {
        self.library_index(name).map(|i| &self.libraries[i])
    }

    /// The active library, for a change. The starter library refuses.
    ///
    /// # Errors
    ///
    /// The active library is the starter library.
    pub fn active_mut(&mut self) -> Result<&mut OpenLibrary, WorkshopError> {
        let lib = &mut self.libraries[self.active];
        if lib.built_in {
            return Err(WorkshopError::new(
                "The starter library cannot be changed. Use Save library as… to make your own copy.",
            ));
        }
        Ok(lib)
    }

    /// Changes the active library with `change`; marks it modified when the
    /// change succeeds.
    ///
    /// # Errors
    ///
    /// The starter library, or whatever `change` returns.
    pub fn edit<T>(
        &mut self,
        change: impl FnOnce(&mut Library) -> Result<T, WorkshopError>,
    ) -> Result<T, WorkshopError> {
        let open = self.active_mut()?;
        let mut copy = open.library.clone();
        let out = change(&mut copy)?;
        open.library = copy;
        open.modified = true;
        Ok(out)
    }

    /// Opens a new, empty library and makes it active.
    ///
    /// # Errors
    ///
    /// An invalid name, or a library of that name is open.
    pub fn new_library(&mut self, name: &str) -> Result<(), WorkshopError> {
        let library = Library::new(name)?;
        self.add_open(library, None, true)
    }

    fn add_open(
        &mut self,
        library: Library,
        path: Option<PathBuf>,
        modified: bool,
    ) -> Result<(), WorkshopError> {
        if self.library_index(&library.name).is_some() {
            return Err(WorkshopError::new(format!(
                "a library named \"{}\" is already open",
                library.name
            )));
        }
        self.libraries.push(OpenLibrary {
            library,
            path,
            modified,
            built_in: false,
        });
        self.active = self.libraries.len() - 1;
        self.save_state();
        Ok(())
    }

    /// Opens a library file and makes it active; one already open from that
    /// file just becomes active.
    ///
    /// # Errors
    ///
    /// The file cannot be read or is not a library, or another open library
    /// has its name.
    pub fn open_library(&mut self, path: &Path) -> Result<(), WorkshopError> {
        if let Some(i) = self
            .libraries
            .iter()
            .position(|l| l.path.as_deref() == Some(path))
        {
            self.active = i;
            self.save_state();
            return Ok(());
        }
        let library = Library::from_json(&read(path)?)?;
        self.add_open(library, Some(path.to_owned()), false)
    }

    /// Writes the active library to its file, or to `path`, which then
    /// becomes its file. Saving the starter library to a path opens the
    /// copy as a new, editable library named "‹name› (copy)".
    ///
    /// # Errors
    ///
    /// No file and no path, or the file cannot be written.
    pub fn save_library(&mut self, path: Option<&Path>) -> Result<PathBuf, WorkshopError> {
        let open = &self.libraries[self.active];
        if open.built_in {
            let path =
                path.ok_or_else(|| WorkshopError::new("Choose a file with Save library as…"))?;
            let mut copy = open.library.clone();
            copy.name = format!("{} (copy)", copy.name);
            copy.note.clear();
            write_atomic(path, &copy.to_json())?;
            self.add_open(copy, Some(path.to_owned()), false)?;
            return Ok(path.to_owned());
        }
        let target = path
            .map(Path::to_owned)
            .or_else(|| open.path.clone())
            .ok_or_else(|| WorkshopError::new("Choose a file with Save library as…"))?;
        write_atomic(&target, &open.library.to_json())?;
        let open = &mut self.libraries[self.active];
        open.path = Some(target.clone());
        open.modified = false;
        self.save_state();
        Ok(target)
    }

    /// Closes a library (the active one when `name` is `None`). Refused for
    /// the starter library, and for unsaved changes unless `force`.
    ///
    /// # Errors
    ///
    /// No such library, the starter library, or unsaved changes.
    pub fn close_library(
        &mut self,
        name: Option<&str>,
        force: bool,
    ) -> Result<String, WorkshopError> {
        let i = match name {
            Some(n) => self
                .library_index(n)
                .ok_or_else(|| WorkshopError::new(format!("no open library named \"{n}\"")))?,
            None => self.active,
        };
        let open = &self.libraries[i];
        if open.built_in {
            return Err(WorkshopError::new("the starter library cannot be closed"));
        }
        if open.modified && !force {
            return Err(WorkshopError::new(format!(
                "{} has changes that are not saved",
                open.library.name
            )));
        }
        let closed = self.libraries.remove(i).library.name;
        if self.active >= i {
            self.active = self.active.saturating_sub(1);
        }
        self.save_state();
        Ok(closed)
    }

    /// Makes an open library active.
    ///
    /// # Errors
    ///
    /// No open library of that name.
    pub fn use_library(&mut self, name: &str) -> Result<(), WorkshopError> {
        self.active = self
            .library_index(name)
            .ok_or_else(|| WorkshopError::new(format!("no open library named \"{name}\"")))?;
        self.save_state();
        Ok(())
    }

    /// Makes the first open library made for `device` active, if any.
    pub fn use_library_for(&mut self, device: &str) -> Option<String> {
        let i = self
            .libraries
            .iter()
            .position(|l| l.library.device.as_deref().is_some_and(|d| same(d, device)))?;
        self.active = i;
        self.save_state();
        Some(self.libraries[i].library.name.clone())
    }

    /// Renames the active library.
    ///
    /// # Errors
    ///
    /// The starter library, an invalid name, or another open library has it.
    pub fn rename_library(&mut self, name: &str) -> Result<(), WorkshopError> {
        let new = Library::new(name)?.name;
        if self.library_index(&new).is_some_and(|i| i != self.active) {
            return Err(WorkshopError::new(format!(
                "a library named \"{new}\" is already open"
            )));
        }
        self.edit(|lib| {
            lib.name = new;
            Ok(())
        })?;
        self.save_state();
        Ok(())
    }

    /// Reads a library file for merging.
    ///
    /// # Errors
    ///
    /// The file cannot be read or is not a library.
    pub fn read_library(path: &Path) -> Result<Library, WorkshopError> {
        Library::from_json(&read(path)?)
    }

    // ---- Test presets ----

    /// Built-in presets, then the user's.
    #[must_use]
    pub fn presets(&self) -> Vec<(Preset, bool)> {
        presets::built_in()
            .into_iter()
            .map(|p| (p, true))
            .chain(self.presets.iter().cloned().map(|p| (p, false)))
            .collect()
    }

    /// Stores a user preset, replacing one of the same name and kind.
    ///
    /// # Errors
    ///
    /// An unknown kind, an empty name, settings that are not an object, or
    /// the name of a built-in preset.
    pub fn save_preset(
        &mut self,
        name: &str,
        kind: &str,
        settings: Value,
    ) -> Result<(), WorkshopError> {
        presets::check_kind(kind)?;
        let name = name.trim();
        if name.is_empty() || name.chars().count() > library::MAX_TEXT_CHARS {
            return Err(WorkshopError::new("a preset name must be 1–60 characters"));
        }
        if !settings.is_object() {
            return Err(WorkshopError::new("preset settings must be a JSON object"));
        }
        if presets::built_in()
            .iter()
            .any(|p| same(&p.name, name) && p.kind == kind)
        {
            return Err(WorkshopError::new("built-in presets cannot be replaced"));
        }
        self.presets
            .retain(|p| !(same(&p.name, name) && p.kind == kind));
        self.presets.push(Preset {
            name: name.to_owned(),
            kind: kind.to_owned(),
            settings,
        });
        self.save_presets();
        Ok(())
    }

    /// Removes a user preset.
    ///
    /// # Errors
    ///
    /// A built-in preset, or no such preset.
    pub fn delete_preset(&mut self, name: &str, kind: &str) -> Result<(), WorkshopError> {
        if presets::built_in()
            .iter()
            .any(|p| same(&p.name, name) && p.kind == kind)
        {
            return Err(WorkshopError::new("Built-in presets cannot be deleted."));
        }
        let before = self.presets.len();
        self.presets
            .retain(|p| !(same(&p.name, name) && p.kind == kind));
        if self.presets.len() == before {
            return Err(WorkshopError::new(format!(
                "no {kind} preset named \"{name}\""
            )));
        }
        self.save_presets();
        Ok(())
    }

    /// Adds the presets of a `.lctests` file (replacing same-named ones);
    /// returns how many.
    ///
    /// # Errors
    ///
    /// The file cannot be read or is not a presets file.
    pub fn import_presets(&mut self, path: &Path) -> Result<usize, WorkshopError> {
        let file = parse_presets(&read(path)?)?;
        let n = file.presets.len();
        for p in file.presets {
            self.save_preset(&p.name, &p.kind, p.settings)?;
        }
        Ok(n)
    }

    /// Writes the user presets as a `.lctests` file.
    ///
    /// # Errors
    ///
    /// The file cannot be written.
    pub fn export_presets(&self, path: &Path) -> Result<usize, WorkshopError> {
        write_atomic(
            path,
            &pretty(&presets::PresetsFile {
                format: presets::FORMAT.to_owned(),
                version: 1,
                presets: self.presets.clone(),
                last: serde_json::Map::new(),
            }),
        )?;
        Ok(self.presets.len())
    }

    /// The settings last used for a pattern kind.
    #[must_use]
    pub fn last_settings(&self, kind: &str) -> Option<&Value> {
        self.last.get(kind)
    }

    /// Remembers the settings last used for a pattern kind.
    pub fn remember_settings(&mut self, kind: &str, settings: Value) {
        if self.last.get(kind) != Some(&settings) {
            self.last.insert(kind.to_owned(), settings);
            self.save_presets();
        }
    }
}

fn parse_profiles(text: &str) -> Result<profile::ProfilesFile, WorkshopError> {
    let file: profile::ProfilesFile = serde_json::from_str(text)
        .map_err(|e| WorkshopError::new(format!("not a LaserCraft device list: {e}")))?;
    if file.format != PROFILES_FORMAT {
        return Err(WorkshopError::new("not a LaserCraft device list"));
    }
    if file.version > 1 {
        return Err(WorkshopError::new(
            "the device list was made by a newer LaserCraft; update LaserCraft",
        ));
    }
    for p in &file.profiles {
        p.validate()
            .map_err(|e| WorkshopError::new(format!("\"{}\": {e}", p.name)))?;
    }
    Ok(file)
}

fn parse_presets(text: &str) -> Result<presets::PresetsFile, WorkshopError> {
    let file: presets::PresetsFile = serde_json::from_str(text)
        .map_err(|e| WorkshopError::new(format!("not a LaserCraft presets file: {e}")))?;
    if file.format != presets::FORMAT {
        return Err(WorkshopError::new("not a LaserCraft presets file"));
    }
    if file.version > 1 {
        return Err(WorkshopError::new(
            "the presets file was made by a newer LaserCraft; update LaserCraft",
        ));
    }
    for p in &file.presets {
        presets::check_kind(&p.kind)?;
    }
    Ok(file)
}

#[cfg(test)]
mod tests;

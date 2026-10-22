//! Material libraries (`docs/spec/materials.md`).

use std::cmp::Ordering;

use lasercraft_project::{Follows, Layer};
use serde::{Deserialize, Serialize};

use crate::WorkshopError;

/// Longest material, group or description text.
pub const MAX_TEXT_CHARS: usize = 60;

/// One material entry: where it sits in the tree and its layer settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    /// Unique in its library.
    pub id: u32,
    /// First tree level ("Plywood").
    pub material: String,
    /// Thickness, mm; absent when a group name is used instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thickness: Option<f64>,
    /// Second tree level when there is no thickness.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// The entry's name in the tree.
    pub description: String,
    /// Layer settings; `color`, `name`, `output`, `show` and `follows` are
    /// ignored.
    pub settings: Layer,
}

/// Where an entry sits: material, thickness or group, description.
#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    /// Material.
    pub material: String,
    /// Thickness, mm.
    pub thickness: Option<f64>,
    /// Group name, without a thickness.
    pub group: Option<String>,
    /// Description.
    pub description: String,
}

fn text(field: &str, value: &str) -> Result<String, WorkshopError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        return Err(WorkshopError::new(format!(
            "{field} must be 1–{MAX_TEXT_CHARS} characters"
        )));
    }
    Ok(value.to_owned())
}

impl Key {
    /// Checks and trims a key: exactly one of thickness and group.
    ///
    /// # Errors
    ///
    /// Empty texts, a thickness that is not above zero, both or neither of
    /// thickness and group.
    pub fn new(
        material: &str,
        thickness: Option<f64>,
        group: Option<&str>,
        description: &str,
    ) -> Result<Self, WorkshopError> {
        let (thickness, group) = match (thickness, group) {
            (Some(t), None) if t.is_finite() && t > 0.0 => (Some(t), None),
            (Some(_), None) => {
                return Err(WorkshopError::new("thickness must be above 0 mm"));
            }
            (None, Some(g)) => (None, Some(text("group", g)?)),
            _ => {
                return Err(WorkshopError::new(
                    "give either a thickness or a group name",
                ));
            }
        };
        Ok(Self {
            material: text("material", material)?,
            thickness,
            group,
            description: text("description", description)?,
        })
    }

    fn same(&self, other: &Self) -> bool {
        let eq = |a: &str, b: &str| a.eq_ignore_ascii_case(b);
        eq(&self.material, &other.material)
            && self.thickness == other.thickness
            && match (&self.group, &other.group) {
                (Some(a), Some(b)) => eq(a, b),
                (None, None) => true,
                _ => false,
            }
            && eq(&self.description, &other.description)
    }
}

impl Material {
    /// The entry's key.
    #[must_use]
    pub fn key(&self) -> Key {
        Key {
            material: self.material.clone(),
            thickness: self.thickness,
            group: self.group.clone(),
            description: self.description.clone(),
        }
    }

    /// The second tree level: "3.00 mm" or the group name.
    #[must_use]
    pub fn level(&self) -> String {
        match (self.thickness, &self.group) {
            (Some(t), _) => format!("{t:.2} mm"),
            (None, Some(g)) => g.clone(),
            (None, None) => String::new(),
        }
    }

    /// How a layer that follows this entry names it.
    #[must_use]
    pub fn follows(&self, library: &str) -> Follows {
        Follows {
            library: library.to_owned(),
            material: self.material.clone(),
            thickness: self.thickness,
            group: self.group.clone(),
            description: self.description.clone(),
        }
    }

    /// Whether `follows` names this entry of `library`.
    #[must_use]
    pub fn is_followed_by(&self, library: &str, follows: &Follows) -> bool {
        follows.library == library
            && self.key().same(&Key {
                material: follows.material.clone(),
                thickness: follows.thickness,
                group: follows.group.clone(),
                description: follows.description.clone(),
            })
    }

    /// The settings written into a layer: the entry's, keeping the layer's
    /// color, name, output and show.
    #[must_use]
    pub fn applied_to(&self, layer: &Layer) -> Layer {
        let mut out = self.settings.clone();
        out.color = layer.color;
        out.name.clone_from(&layer.name);
        out.output = layer.output;
        out.show = layer.show;
        out.follows.clone_from(&layer.follows);
        out
    }

    fn cmp_tree(&self, other: &Self) -> Ordering {
        let lower = |s: &str| s.to_lowercase();
        lower(&self.material)
            .cmp(&lower(&other.material))
            .then_with(|| match (self.thickness, other.thickness) {
                (Some(a), Some(b)) => a.total_cmp(&b),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => lower(self.group.as_deref().unwrap_or_default())
                    .cmp(&lower(other.group.as_deref().unwrap_or_default())),
            })
            .then_with(|| lower(&self.description).cmp(&lower(&other.description)))
    }
}

/// Settings stored in an entry: the layer's, normalised so files do not
/// carry a color, a layer name or a follow mark.
#[must_use]
pub fn stored(layer: &Layer, description: &str) -> Layer {
    let mut s = layer.clone();
    s.color = 0;
    description.clone_into(&mut s.name);
    s.output = true;
    s.show = true;
    s.follows = None;
    s
}

/// A material library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Library {
    /// The library's name.
    pub name: String,
    /// A note shown with the library (the starter library's advice).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
    /// The device profile it was made for.
    #[serde(default)]
    pub device: Option<String>,
    next_id: u32,
    entries: Vec<Material>,
}

impl Library {
    /// An empty library.
    ///
    /// # Errors
    ///
    /// An empty or overlong name.
    pub fn new(name: &str) -> Result<Self, WorkshopError> {
        Ok(Self {
            name: text("library name", name)?,
            note: String::new(),
            device: None,
            next_id: 1,
            entries: Vec::new(),
        })
    }

    /// Entries in tree order.
    #[must_use]
    pub fn entries(&self) -> &[Material] {
        &self.entries
    }

    /// The entry with this id.
    ///
    /// # Errors
    ///
    /// No such entry.
    pub fn entry(&self, id: u32) -> Result<&Material, WorkshopError> {
        self.entries
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| WorkshopError::new(format!("no material entry {id} in {}", self.name)))
    }

    fn entry_mut(&mut self, id: u32) -> Result<&mut Material, WorkshopError> {
        let name = self.name.clone();
        self.entries
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or_else(|| WorkshopError::new(format!("no material entry {id} in {name}")))
    }

    /// The entry a layer's `follows` names, if it is in this library.
    #[must_use]
    pub fn followed(&self, follows: &Follows) -> Option<&Material> {
        self.entries
            .iter()
            .find(|e| e.is_followed_by(&self.name, follows))
    }

    fn clash(&self, key: &Key, except: Option<u32>) -> Result<(), WorkshopError> {
        if self
            .entries
            .iter()
            .any(|e| Some(e.id) != except && e.key().same(key))
        {
            return Err(WorkshopError::new(format!(
                "{} already has \"{}\" for {} {}",
                self.name,
                key.description,
                key.material,
                Material {
                    id: 0,
                    material: String::new(),
                    thickness: key.thickness,
                    group: key.group.clone(),
                    description: String::new(),
                    settings: Layer::new(0),
                }
                .level()
            )));
        }
        Ok(())
    }

    fn sort(&mut self) {
        self.entries.sort_by(Material::cmp_tree);
    }

    /// Adds an entry; returns its id.
    ///
    /// # Errors
    ///
    /// The key is taken, or the settings are invalid.
    pub fn add(&mut self, key: Key, settings: &Layer) -> Result<u32, WorkshopError> {
        self.clash(&key, None)?;
        let settings = stored(settings, &key.description);
        settings
            .validate()
            .map_err(|e| WorkshopError::new(e.to_string()))?;
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(Material {
            id,
            material: key.material,
            thickness: key.thickness,
            group: key.group,
            description: key.description,
            settings,
        });
        self.sort();
        Ok(id)
    }

    /// Replaces an entry's settings.
    ///
    /// # Errors
    ///
    /// No such entry, or invalid settings.
    pub fn set_settings(&mut self, id: u32, settings: &Layer) -> Result<(), WorkshopError> {
        let entry = self.entry_mut(id)?;
        let settings = stored(settings, &entry.description);
        settings
            .validate()
            .map_err(|e| WorkshopError::new(e.to_string()))?;
        entry.settings = settings;
        Ok(())
    }

    /// Changes an entry's key.
    ///
    /// # Errors
    ///
    /// No such entry, or the key is taken.
    pub fn describe(&mut self, id: u32, key: Key) -> Result<(), WorkshopError> {
        self.entry(id)?;
        self.clash(&key, Some(id))?;
        let entry = self.entry_mut(id)?;
        entry.settings.name.clone_from(&key.description);
        entry.material = key.material;
        entry.thickness = key.thickness;
        entry.group = key.group;
        entry.description = key.description;
        self.sort();
        Ok(())
    }

    /// Copies an entry as "‹description› (copy)"; returns the new id.
    ///
    /// # Errors
    ///
    /// No such entry.
    pub fn duplicate(&mut self, id: u32) -> Result<u32, WorkshopError> {
        let source = self.entry(id)?.clone();
        let mut n = 1;
        loop {
            let description = if n == 1 {
                format!("{} (copy)", source.description)
            } else {
                format!("{} (copy {n})", source.description)
            };
            let key = Key {
                description,
                ..source.key()
            };
            if self.clash(&key, None).is_ok() {
                return self.add(key, &source.settings);
            }
            n += 1;
        }
    }

    /// Removes an entry and returns it.
    ///
    /// # Errors
    ///
    /// No such entry.
    pub fn remove(&mut self, id: u32) -> Result<Material, WorkshopError> {
        let at = self
            .entries
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| {
                WorkshopError::new(format!("no material entry {id} in {}", self.name))
            })?;
        Ok(self.entries.remove(at))
    }

    /// Adds every entry of `other` whose key is free; returns how many were
    /// added and how many skipped.
    pub fn merge(&mut self, other: &Self) -> (usize, usize) {
        let (mut added, mut skipped) = (0, 0);
        for e in &other.entries {
            if self.add(e.key(), &e.settings).is_ok() {
                added += 1;
            } else {
                skipped += 1;
            }
        }
        (added, skipped)
    }

    /// Checks a library read from a file.
    ///
    /// # Errors
    ///
    /// Bad texts, keys, settings, repeated ids or keys.
    pub fn validate(&self) -> Result<(), WorkshopError> {
        text("library name", &self.name)?;
        let mut seen = std::collections::HashSet::new();
        for (i, e) in self.entries.iter().enumerate() {
            let key = Key::new(&e.material, e.thickness, e.group.as_deref(), &e.description)?;
            if !seen.insert(e.id) || e.id == 0 || e.id >= self.next_id {
                return Err(WorkshopError::new(format!(
                    "entry id {} is not valid",
                    e.id
                )));
            }
            if self.entries[..i].iter().any(|o| o.key().same(&key)) {
                return Err(WorkshopError::new(format!(
                    "\"{}\" appears twice",
                    e.description
                )));
            }
            e.settings
                .validate()
                .map_err(|err| WorkshopError::new(format!("\"{}\": {err}", e.description)))?;
        }
        Ok(())
    }

    /// The `.lcmat` text.
    #[must_use]
    pub fn to_json(&self) -> String {
        let file = LibraryFile {
            format: FORMAT.to_owned(),
            version: VERSION,
            library: self.clone(),
        };
        let mut text = serde_json::to_string_pretty(&file).unwrap_or_default();
        text.push('\n');
        text
    }

    /// Reads `.lcmat` text.
    ///
    /// # Errors
    ///
    /// Not a library file, a newer version, or invalid contents.
    pub fn from_json(text: &str) -> Result<Self, WorkshopError> {
        let file: LibraryFile = serde_json::from_str(text)
            .map_err(|e| WorkshopError::new(format!("not a LaserCraft material library: {e}")))?;
        if file.format != FORMAT {
            return Err(WorkshopError::new("not a LaserCraft material library"));
        }
        if file.version > VERSION {
            return Err(WorkshopError::new(
                "this material library was made by a newer LaserCraft; update LaserCraft to open it",
            ));
        }
        let mut library = file.library;
        library.validate()?;
        library.sort();
        Ok(library)
    }

    /// An indented tree with ids, as `material.list` prints it.
    #[must_use]
    pub fn tree(&self) -> String {
        let mut out = String::new();
        let mut last: (String, String) = (String::new(), String::new());
        for e in &self.entries {
            if e.material != last.0 {
                out.push_str(&e.material);
                out.push('\n');
                last = (e.material.clone(), String::new());
            }
            if e.level() != last.1 {
                out.push_str("  ");
                out.push_str(&e.level());
                out.push('\n');
                last.1 = e.level();
            }
            out.push_str(&format!(
                "    {:>3}  {} ({}, {} mm/s, {}%)\n",
                e.id, e.description, e.settings.mode, e.settings.speed, e.settings.max_power
            ));
        }
        out
    }
}

/// `format` of a library file.
pub const FORMAT: &str = "lasercraft-materials";
/// Current library file version.
pub const VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct LibraryFile {
    format: String,
    version: u32,
    #[serde(flatten)]
    library: Library,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(material: &str, thickness: f64, description: &str) -> Key {
        Key::new(material, Some(thickness), None, description).unwrap()
    }

    #[test]
    fn entries_sort_by_material_thickness_and_description() {
        let mut lib = Library::new("Shop").unwrap();
        let layer = Layer::new(3);
        lib.add(key("Plywood", 6.0, "Cut"), &layer).unwrap();
        lib.add(key("plywood", 3.0, "Engrave"), &layer).unwrap();
        lib.add(key("Plywood", 3.0, "Cut"), &layer).unwrap();
        lib.add(
            Key::new("Plywood", None, Some("Any"), "Score").unwrap(),
            &layer,
        )
        .unwrap();
        lib.add(key("Acrylic", 3.0, "Cut"), &layer).unwrap();
        let order: Vec<String> = lib
            .entries()
            .iter()
            .map(|e| format!("{} {} {}", e.material, e.level(), e.description))
            .collect();
        assert_eq!(
            order,
            [
                "Acrylic 3.00 mm Cut",
                "Plywood 3.00 mm Cut",
                "plywood 3.00 mm Engrave",
                "Plywood 6.00 mm Cut",
                "Plywood Any Score"
            ]
        );
        assert!(
            lib.add(key("PLYWOOD", 3.0, "cut"), &layer).is_err(),
            "same key"
        );
        // Stored settings carry no color or layer name.
        assert_eq!(lib.entries()[0].settings.color, 0);
        assert_eq!(lib.entries()[0].settings.name, "Cut");
    }

    #[test]
    fn keys_need_one_of_thickness_and_group() {
        assert!(Key::new("Wood", None, None, "Cut").is_err());
        assert!(Key::new("Wood", Some(3.0), Some("Any"), "Cut").is_err());
        assert!(Key::new("Wood", Some(0.0), None, "Cut").is_err());
        assert!(Key::new(" ", Some(3.0), None, "Cut").is_err());
    }

    #[test]
    fn files_round_trip_and_reject_damage() {
        let mut lib = Library::new("Shop").unwrap();
        let mut layer = Layer::new(4);
        layer.speed = 33.0;
        let id = lib.add(key("Birch", 3.0, "Cut"), &layer).unwrap();
        lib.duplicate(id).unwrap();
        lib.duplicate(id).unwrap();
        assert!(
            lib.entries()
                .iter()
                .any(|e| e.description == "Cut (copy 2)")
        );
        let text = lib.to_json();
        assert_eq!(Library::from_json(&text).unwrap(), lib);
        assert!(Library::from_json(&text.replace("\"version\": 1", "\"version\": 9")).is_err());
        assert!(Library::from_json("{}").is_err());
        let twice = text.replace("Cut (copy)", "Cut");
        assert!(Library::from_json(&twice).is_err());
    }

    #[test]
    fn follows_matches_case_insensitively_and_applies_keep_the_layer_identity() {
        let mut lib = Library::new("Shop").unwrap();
        let mut settings = Layer::new(0);
        settings.speed = 42.0;
        let id = lib.add(key("Birch", 3.0, "Cut"), &settings).unwrap();
        let entry = lib.entry(id).unwrap();
        let mut follows = entry.follows("Shop");
        follows.material = "BIRCH".into();
        assert!(lib.followed(&follows).is_some());
        follows.library = "Other".into();
        assert!(lib.followed(&follows).is_none());
        let mut target = Layer::new(7);
        target.name = "Frame".into();
        target.output = false;
        let applied = entry.applied_to(&target);
        assert_eq!(
            (
                applied.color,
                applied.name.as_str(),
                applied.output,
                applied.speed
            ),
            (7, "Frame", false, 42.0)
        );
    }
}

use std::path::PathBuf;

use lasercraft_project::Layer;
use serde_json::json;

use super::*;

/// A fresh folder under the system temp folder.
fn folder(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("lasercraft-workshop-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn profiles_are_saved_at_once_and_load_back() {
    let dir = folder("profiles");
    let (mut ws, messages) = Workshop::load(&dir);
    assert!(messages.is_empty());
    ws.add_profile(DeviceProfile::new("Diode").unwrap())
        .unwrap();
    assert_eq!(
        ws.default_profile(),
        Some("Diode"),
        "the first becomes the default"
    );
    ws.add_profile(DeviceProfile::new("CO2").unwrap()).unwrap();
    assert!(
        ws.add_profile(DeviceProfile::new("diode").unwrap())
            .is_err()
    );
    ws.set_profile("co2", "width", "600").unwrap();
    ws.rename_profile("Diode", "Shop diode").unwrap();
    assert_eq!(ws.default_profile(), Some("Shop diode"));
    ws.duplicate_profile("CO2", "CO2 spare").unwrap();
    ws.delete_profile("CO2 spare").unwrap();
    let (back, messages) = Workshop::load(&dir);
    assert!(messages.is_empty(), "{messages:?}");
    assert_eq!(back.profiles(), ws.profiles());
    assert_eq!(back.default_profile(), Some("Shop diode"));
    assert_eq!(back.profile("CO2").unwrap().width, 600.0);
    // Export and import with a clashing name.
    let file = dir.join("co2.lcdevice");
    ws.export_profile("CO2", &file).unwrap();
    assert_eq!(ws.import_profile(&file).unwrap(), "CO2 (2)");
    ws.delete_profile("Shop diode").unwrap();
    assert_eq!(ws.default_profile(), None);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_damaged_profiles_file_is_reported_not_fatal() {
    let dir = folder("damaged");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(PROFILES_FILE), "{not json").unwrap();
    let (ws, messages) = Workshop::load(&dir);
    assert!(ws.profiles().is_empty());
    assert_eq!(messages.len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_starter_library_is_read_only_and_saves_as_a_copy() {
    let dir = folder("starter");
    let (mut ws, _) = Workshop::load(&dir);
    assert!(ws.active().built_in);
    let refused = ws.edit(|lib| lib.add(Key::new("Oak", Some(3.0), None, "Cut")?, &Layer::new(0)));
    assert!(refused.unwrap_err().0.contains("cannot be changed"));
    assert!(ws.close_library(None, true).is_err());
    let path = dir.join("mine.lcmat");
    ws.save_library(Some(&path)).unwrap();
    assert_eq!(ws.active().library.name, "LaserCraft starter (copy)");
    assert!(!ws.active().built_in);
    let id = ws
        .edit(|lib| lib.add(Key::new("Oak", Some(3.0), None, "Cut")?, &Layer::new(2)))
        .unwrap();
    assert!(ws.active().modified);
    assert!(ws.close_library(None, false).is_err(), "unsaved changes");
    ws.save_library(None).unwrap();
    assert!(!ws.active().modified);
    // The open libraries come back at the next start, the copy active.
    let (back, messages) = Workshop::load(&dir);
    assert!(messages.is_empty(), "{messages:?}");
    assert_eq!(back.libraries().len(), 2);
    assert_eq!(back.active().library.entry(id).unwrap().material, "Oak");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn libraries_tied_to_a_device_become_active_with_it() {
    let mut ws = Workshop::in_memory();
    ws.new_library("CO2 shop").unwrap();
    ws.edit(|lib| {
        lib.device = Some("CO2".into());
        Ok(())
    })
    .unwrap();
    ws.use_library(starter::NAME).unwrap();
    assert_eq!(ws.use_library_for("co2").as_deref(), Some("CO2 shop"));
    assert!(ws.use_library_for("Diode").is_none());
    assert!(ws.new_library("co2 SHOP").is_err(), "same name open");
}

#[test]
fn presets_keep_built_ins_and_round_trip() {
    let dir = folder("presets");
    let (mut ws, _) = Workshop::load(&dir);
    let builtin = ws.presets().len();
    ws.save_preset("Mine", "speed_power", json!({"rows": {"count": 4}}))
        .unwrap();
    ws.save_preset("Mine", "speed_power", json!({"rows": {"count": 5}}))
        .unwrap();
    assert_eq!(ws.presets().len(), builtin + 1, "same name replaces");
    assert!(ws.save_preset("Mine", "focus", json!({})).is_err());
    assert!(
        ws.save_preset("Diode engraving", "speed_power", json!({}))
            .is_err()
    );
    assert!(ws.delete_preset("Diode cutting", "speed_power").is_err());
    ws.remember_settings("interval", json!({"patches": 3}));
    let file = dir.join("mine.lctests");
    assert_eq!(ws.export_presets(&file).unwrap(), 1);
    ws.delete_preset("Mine", "speed_power").unwrap();
    assert_eq!(ws.import_presets(&file).unwrap(), 1);
    let (back, _) = Workshop::load(&dir);
    assert_eq!(back.presets(), ws.presets());
    assert_eq!(back.last_settings("interval"), Some(&json!({"patches": 3})));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_config_dir_honours_the_override() {
    // Only the override is checked: other variables belong to the user.
    if let Some(dir) = std::env::var_os("LASERCRAFT_CONFIG_DIR") {
        assert_eq!(config_dir(), Some(PathBuf::from(dir)));
    } else {
        assert!(
            config_dir().is_none_or(|d| d.ends_with("lasercraft") || d.ends_with("LaserCraft"))
        );
    }
}

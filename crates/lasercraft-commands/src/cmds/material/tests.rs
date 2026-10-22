use crate::{Registry, Session};

fn session() -> Session {
    let mut s = Session::new(
        lasercraft_project::Project::default(),
        Registry::with_builtin(),
    );
    for line in [
        "layer.set 3 speed 42",
        "layer.set 3 max_power 55",
        "layer.set 3 name Frame",
        "library.new Shop",
    ] {
        s.run_line(line).unwrap();
    }
    s
}

fn layer(s: &Session, color: u8) -> lasercraft_project::Layer {
    s.project().layer(color).unwrap().clone()
}

#[test]
fn entries_are_stored_applied_and_undone() {
    let mut s = session();
    s.run_line("material.add Birch 3 Cut 3").unwrap();
    s.run_line("material.add Birch - Engrave 3 group=Any")
        .unwrap();
    let list = s.run_line("material.list").unwrap().message;
    assert!(
        list.contains("Birch") && list.contains("3.00 mm") && list.contains("Any"),
        "{list}"
    );
    assert!(
        s.run_line("material.add Birch 3 cut 3").is_err(),
        "same key"
    );
    assert!(
        s.run_line("material.add Birch - X 3").is_err(),
        "no thickness, no group"
    );
    // Apply copies the settings, keeping the layer's own name.
    let before = s.project().clone();
    s.run_line("material.apply 1 7").unwrap();
    let applied = layer(&s, 7);
    assert_eq!(
        (applied.speed, applied.max_power, applied.name.as_str()),
        (42.0, 55.0, "Layer 07")
    );
    assert!(applied.follows.is_none());
    s.run_line("edit.undo").unwrap();
    assert_eq!(s.project(), &before);
    // Library edits are not undo steps.
    assert_eq!(s.workshop().active().library.entries().len(), 2);
}

#[test]
fn followers_are_read_only_and_track_the_entry() {
    let mut s = session();
    s.run_line("material.add Birch 3 Cut 3").unwrap();
    s.run_line("material.follow 1 9").unwrap();
    let followed = layer(&s, 9);
    assert_eq!(followed.follows.as_ref().unwrap().description, "Cut");
    assert_eq!(followed.speed, 42.0);
    // Settings are refused; name, output and show are not.
    assert!(
        s.run_line("layer.set 9 speed 10")
            .unwrap_err()
            .to_string()
            .contains("Stop following")
    );
    assert!(s.run_line("layer.copy_settings 3 9").is_err());
    s.run_line("layer.set 9 name Edge").unwrap();
    // Changing the entry changes the follower, undoably for the project.
    let before = s.project().clone();
    s.run_line("material.set 1 speed 12").unwrap();
    assert_eq!(layer(&s, 9).speed, 12.0);
    assert_eq!(layer(&s, 9).name, "Edge");
    s.run_line("edit.undo").unwrap();
    assert_eq!(s.project(), &before);
    assert_eq!(
        s.workshop()
            .active()
            .library
            .entry(1)
            .unwrap()
            .settings
            .speed,
        12.0,
        "the entry keeps its change"
    );
    // Renaming the entry moves the follow mark with it.
    s.run_line("material.describe 1 Birch 3 \"Clean cut\"")
        .unwrap();
    assert_eq!(layer(&s, 9).follows.unwrap().description, "Clean cut");
    // Update from another layer.
    s.run_line("layer.set 4 speed 77").unwrap();
    s.run_line("material.update 1 4").unwrap();
    assert_eq!(layer(&s, 9).speed, 77.0);
    // Deleting stops following; settings stay.
    s.run_line("material.delete 1").unwrap();
    let after = layer(&s, 9);
    assert!(after.follows.is_none() && after.speed == 77.0);
    s.run_line("edit.undo").unwrap();
    assert!(
        layer(&s, 9).follows.is_some(),
        "undo puts the follow mark back"
    );
    s.run_line("material.unfollow 9").unwrap();
    s.run_line("layer.set 9 speed 5").unwrap();
    assert!(s.run_line("material.unfollow 9").is_err());
}

#[test]
fn the_starter_library_is_read_only_but_followable() {
    let mut s = session();
    s.run_line("library.use \"LaserCraft starter\"").unwrap();
    let list = s.run_line("material.list").unwrap().message;
    assert!(list.contains("Plywood"), "{list}");
    assert!(s.run_line("material.add Oak 3 Cut 3").is_err());
    assert!(s.run_line("material.set 1 speed 5").is_err());
    let before = s.project().clone();
    s.run_line("material.follow 1 2").unwrap();
    assert!(layer(&s, 2).follows.is_some());
    s.run_line("edit.undo").unwrap();
    assert_eq!(s.project(), &before);
}

#[test]
fn libraries_save_open_merge_and_close() {
    let dir = std::env::temp_dir().join(format!("lasercraft-material-cmds-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("shop.lcmat");
    let quoted = |p: &std::path::Path| format!("\"{}\"", p.display());
    let mut s = session();
    s.run_line("material.add Birch 3 Cut 3").unwrap();
    assert!(s.run_line("library.save").is_err(), "no file yet");
    assert!(s.run_line("library.close").is_err(), "unsaved changes");
    s.run_line(&format!("library.save {}", quoted(&file)))
        .unwrap();
    s.run_line("library.set_device Diode").unwrap();
    s.run_line("library.close Shop force").unwrap();
    s.run_line(&format!("library.open {}", quoted(&file)))
        .unwrap();
    assert_eq!(s.workshop().active().library.name, "Shop");
    assert_eq!(
        s.workshop().active().library.device,
        None,
        "the forced close dropped it"
    );
    s.run_line("library.new Other").unwrap();
    let merged = s
        .run_line(&format!("library.merge {}", quoted(&file)))
        .unwrap()
        .message;
    assert!(merged.contains("1 entries added"), "{merged}");
    s.run_line("library.rename \"Other two\"").unwrap();
    assert!(s.run_line("library.rename Shop").is_err(), "name in use");
    let _ = std::fs::remove_dir_all(&dir);
}

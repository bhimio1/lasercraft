//! The starter library: LaserCraft's own conservative starting points for
//! a typical 5–10 W diode laser (`docs/spec/materials.md`). The values come
//! from how much energy each material needs relative to the others, not
//! from any other product's library. They are meant to be refined with a
//! Speed and power test.

use lasercraft_project::{Layer, Mode};

use crate::library::{Key, Library};

/// The starter library's name.
pub const NAME: &str = "LaserCraft starter";

/// One starter entry: material, thickness (or group), description, mode,
/// speed (mm/s), max power (%), passes, interval (mm), air assist.
type Row = (
    &'static str,
    Level,
    &'static str,
    Mode,
    f64,
    f64,
    u32,
    f64,
    bool,
);

#[derive(Clone, Copy)]
enum Level {
    Mm(f64),
    Group(&'static str),
}

#[rustfmt::skip]
const ROWS: [Row; 14] = [
    ("Plywood", Level::Mm(3.0), "Engrave", Mode::Fill, 150.0, 30.0, 1, 0.1, false),
    ("Plywood", Level::Mm(3.0), "Score", Mode::Line, 30.0, 50.0, 1, 0.1, true),
    ("Plywood", Level::Mm(3.0), "Cut", Mode::Line, 4.0, 100.0, 3, 0.1, true),
    ("Basswood", Level::Mm(3.0), "Engrave", Mode::Fill, 180.0, 25.0, 1, 0.1, false),
    ("Basswood", Level::Mm(3.0), "Cut", Mode::Line, 5.0, 100.0, 2, 0.1, true),
    ("MDF", Level::Mm(3.0), "Engrave", Mode::Fill, 120.0, 35.0, 1, 0.1, false),
    ("MDF", Level::Mm(3.0), "Cut", Mode::Line, 3.0, 100.0, 4, 0.1, true),
    ("Cardboard", Level::Mm(2.0), "Cut", Mode::Line, 10.0, 80.0, 1, 0.1, true),
    ("Leather", Level::Mm(2.0), "Engrave", Mode::Fill, 150.0, 20.0, 1, 0.1, false),
    ("Leather", Level::Mm(2.0), "Score", Mode::Line, 40.0, 40.0, 1, 0.1, false),
    ("Aluminum", Level::Group("Black anodized"), "Engrave", Mode::Fill, 100.0, 70.0, 1, 0.08, false),
    ("Slate", Level::Group("Tile"), "Engrave", Mode::Fill, 120.0, 60.0, 1, 0.1, false),
    ("Paper", Level::Mm(0.2), "Cut", Mode::Line, 30.0, 40.0, 1, 0.1, false),
    ("Paper", Level::Mm(0.2), "Engrave", Mode::Fill, 250.0, 10.0, 1, 0.1, false),
];

/// The starter library.
#[must_use]
pub fn library() -> Library {
    let mut lib = Library::new(NAME).unwrap_or_else(|_| unreachable!("valid name"));
    lib.note = "Starting points only: run a Speed and power test on your machine first.".to_owned();
    for (material, level, description, mode, speed, power, passes, interval, air) in ROWS {
        let (thickness, group) = match level {
            Level::Mm(t) => (Some(t), None),
            Level::Group(g) => (None, Some(g)),
        };
        let mut settings = Layer::new(0);
        settings.mode = mode;
        settings.speed = speed;
        settings.max_power = power;
        settings.passes = passes;
        settings.interval = interval;
        settings.air = air;
        let key = Key::new(material, thickness, group, description)
            .unwrap_or_else(|_| unreachable!("valid starter key"));
        lib.add(key, &settings)
            .unwrap_or_else(|_| unreachable!("valid starter entry"));
    }
    lib
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_starter_library_is_valid_and_complete() {
        let lib = super::library();
        lib.validate().unwrap();
        assert_eq!(lib.entries().len(), super::ROWS.len());
        assert!(lib.entries().iter().all(|e| e.settings.max_power <= 100.0));
    }
}

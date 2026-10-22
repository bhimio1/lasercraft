//! Keyboard shortcuts as data: a text file mapping key chords to invocations.

use std::collections::BTreeMap;
use std::fmt;

use lasercraft_commands::Invocation;

/// The keymap compiled into the app. Users may override it with their own file.
pub const DEFAULT_KEYMAP: &str = include_str!("../keymap/default.keymap");

/// A key with modifiers, such as `Ctrl+Shift+Z`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Chord {
    /// Ctrl (Cmd on macOS).
    pub ctrl: bool,
    /// Shift.
    pub shift: bool,
    /// Alt / Option.
    pub alt: bool,
    /// egui key name, for example `Space`, `F5`, `Z`.
    pub key: String,
}

impl Chord {
    /// Parses `Ctrl+Shift+Z`-style text. Modifier names are case-insensitive.
    ///
    /// # Errors
    ///
    /// Returns a description of the problem for unknown modifiers or keys.
    pub fn parse(text: &str) -> Result<Self, String> {
        let parts: Vec<&str> = text.split('+').map(str::trim).collect();
        // `Ctrl++` binds the plus key.
        let (mods, key) = match parts.as_slice() {
            [mods @ .., "", ""] => (mods, "Plus"),
            [mods @ .., key] => (mods, *key),
            [] => return Err("empty chord".to_owned()),
        };
        let mut chord = Self {
            ctrl: false,
            shift: false,
            alt: false,
            key: String::new(),
        };
        for m in mods {
            match m.to_ascii_lowercase().as_str() {
                "ctrl" | "cmd" | "command" => chord.ctrl = true,
                "shift" => chord.shift = true,
                "alt" | "option" => chord.alt = true,
                other => return Err(format!("unknown modifier `{other}`")),
            }
        }
        let key = egui::Key::from_name(key).ok_or_else(|| format!("unknown key `{key}`"))?;
        chord.key = key.name().to_owned();
        Ok(chord)
    }

    /// Builds a chord from an egui key event.
    #[must_use]
    pub fn from_event(key: egui::Key, modifiers: egui::Modifiers) -> Self {
        Self {
            ctrl: modifiers.command || modifiers.ctrl,
            shift: modifiers.shift,
            alt: modifiers.alt,
            key: key.name().to_owned(),
        }
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ctrl {
            f.write_str("Ctrl+")?;
        }
        if self.shift {
            f.write_str("Shift+")?;
        }
        if self.alt {
            f.write_str("Alt+")?;
        }
        f.write_str(&self.key)
    }
}

impl Chord {
    /// How menus and tooltips show the chord (`docs/spec/style/naming.md`,
    /// "Chords"): key symbols instead of key names ("Ctrl+=", "PgUp",
    /// "Del"), and Cmd / Option on macOS.
    #[must_use]
    pub fn label(&self) -> String {
        let mac = cfg!(target_os = "macos");
        let mut out = String::new();
        if self.ctrl {
            out.push_str(if mac { "Cmd+" } else { "Ctrl+" });
        }
        if self.shift {
            out.push_str("Shift+");
        }
        if self.alt {
            out.push_str(if mac { "Option+" } else { "Alt+" });
        }
        let key = egui::Key::from_name(&self.key);
        out.push_str(match key {
            Some(egui::Key::PageUp) => "PgUp",
            Some(egui::Key::PageDown) => "PgDn",
            Some(egui::Key::Delete) => "Del",
            Some(egui::Key::Escape) => "Esc",
            Some(egui::Key::Minus) => "-",
            Some(k) => k.symbol_or_name(),
            None => &self.key,
        });
        out
    }
}

/// Chord → invocation bindings.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Keymap {
    bindings: BTreeMap<Chord, Invocation>,
}

impl Keymap {
    /// Parses keymap text. Blank lines and `#` comments are ignored.
    ///
    /// # Errors
    ///
    /// Returns `line N: problem` for the first malformed line.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut bindings = BTreeMap::new();
        for (number, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fail = |reason: String| format!("line {}: {reason}", number + 1);
            let (chord, action) = line
                .split_once('=')
                .ok_or_else(|| fail("expected `<chord> = <invocation>`".to_owned()))?;
            let chord = Chord::parse(chord.trim()).map_err(fail)?;
            let invocation = Invocation::parse(action).map_err(|e| fail(e.to_string()))?;
            if bindings.insert(chord.clone(), invocation).is_some() {
                return Err(fail(format!("`{chord}` is bound twice")));
            }
        }
        Ok(Self { bindings })
    }

    /// The built-in keymap.
    #[must_use]
    pub fn default_keymap() -> Self {
        Self::parse(DEFAULT_KEYMAP).unwrap_or_default()
    }

    /// The invocation bound to a chord.
    #[must_use]
    pub fn lookup(&self, chord: &Chord) -> Option<&Invocation> {
        self.bindings.get(chord)
    }

    /// The first chord bound to an invocation, for menus and tooltips.
    #[must_use]
    pub fn chord_for(&self, invocation: &str) -> Option<&Chord> {
        self.bindings
            .iter()
            .find(|(_, inv)| inv.to_string() == invocation)
            .map(|(chord, _)| chord)
    }

    /// Every binding in chord order.
    pub fn iter(&self) -> impl Iterator<Item = (&Chord, &Invocation)> {
        self.bindings.iter()
    }
}

#[cfg(test)]
mod tests {
    use lasercraft_commands::{Registry, SESSION_COMMANDS};

    use super::*;

    fn bound(keymap: &Keymap, chord: &str) -> String {
        keymap
            .lookup(&Chord::parse(chord).unwrap())
            .unwrap()
            .to_string()
    }

    #[test]
    fn default_keymap_binds_file_and_edit_actions() {
        let keymap = Keymap::parse(DEFAULT_KEYMAP).unwrap();
        for (chord, action) in [
            ("Ctrl+N", "ui.new"),
            ("Ctrl+O", "ui.open"),
            ("Ctrl+S", "ui.save"),
            ("Ctrl+Shift+S", "ui.save_as"),
            ("Ctrl+Z", "edit.undo"),
            ("Ctrl+Shift+Z", "edit.redo"),
            ("Ctrl+Q", "ui.quit"),
        ] {
            assert_eq!(bound(&keymap, chord), action, "{chord}");
        }
        // Not a published default.
        assert!(keymap.lookup(&Chord::parse("Ctrl+Y").unwrap()).is_none());
    }

    #[test]
    fn every_binding_names_a_command_or_ui_action() {
        let registry = Registry::with_builtin();
        for (chord, invocation) in Keymap::default_keymap().iter() {
            let name = invocation.name.as_str();
            let known = registry.get(name).is_some()
                || SESSION_COMMANDS.iter().any(|(n, ..)| *n == name)
                || crate::UI_ACTIONS.contains(&name);
            assert!(known, "{chord} is bound to unknown `{name}`");
        }
    }

    #[test]
    fn chords_normalize() {
        let a = Chord::parse("Shift + Ctrl + s").unwrap();
        let b = Chord::parse("Ctrl+Shift+S").unwrap();
        assert_eq!(a, b);
        assert_eq!(a.to_string(), "Ctrl+Shift+S");
        if !cfg!(target_os = "macos") {
            for (text, label) in [
                ("Ctrl+Equals", "Ctrl+="),
                ("Ctrl+Minus", "Ctrl+-"),
                ("Delete", "Del"),
                ("Escape", "Esc"),
                ("Ctrl+PageUp", "Ctrl+PgUp"),
                ("Alt+Plus", "Alt++"),
                ("Ctrl+Backtick", "Ctrl+`"),
                ("Period", "."),
                ("Ctrl+Shift+S", "Ctrl+Shift+S"),
            ] {
                assert_eq!(Chord::parse(text).unwrap().label(), label, "{text}");
            }
        }
        assert_eq!(Chord::parse("Ctrl++").unwrap().key, "Plus");
        assert_eq!(
            Chord::from_event(egui::Key::Z, egui::Modifiers::COMMAND),
            Chord::parse("Ctrl+Z").unwrap()
        );
    }

    #[test]
    fn errors_name_the_line() {
        assert_eq!(
            Keymap::parse("# ok\nHyper+A = x.y").unwrap_err(),
            "line 2: unknown modifier `hyper`"
        );
        assert!(Keymap::parse("Ctrl+Z = a.b\nctrl+z = c.d").is_err());
        assert!(Keymap::parse("Ctrl+Z edit.undo").is_err());
        assert!(Keymap::parse("NotAKey = a.b").is_err());
    }
}

//! LaserCraft's egui front end.
//!
//! The UI holds no business logic: it draws the project it is given and turns
//! clicks, menu picks, shortcuts and console lines into [`Action`]s. The app
//! runs those through the command session (`docs/spec/ui-shell.md`).

pub mod keymap;
mod shell;

pub use shell::{Action, Shell, ShellView};

/// The `ui.*` names a keymap may bind besides registry commands. They need
/// something only the front end has (a path prompt, the saved-file path).
pub const UI_ACTIONS: [&str; 5] = ["ui.new", "ui.open", "ui.save", "ui.save_as", "ui.quit"];

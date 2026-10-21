//! LaserCraft's egui front end.
//!
//! The UI holds no business logic: it draws the project it is given and turns
//! clicks, menu picks, shortcuts and console lines into [`Action`]s. The app
//! runs those through the command session (`docs/spec/ui-shell.md`).

mod canvas;
pub mod colors;
mod dialogs;
pub mod interaction;
pub mod keymap;
mod layers_panel;
mod preview;
mod shell;
pub mod view;

pub use preview::{Preview, PreviewKind, PreviewMove};
pub use shell::{Action, Shell, ShellView};

/// The `ui.*` and `view.*` names a keymap may bind besides registry
/// commands. They need something only the front end has (a path prompt, the
/// saved-file path, the pointer, the tool, the zoom).
pub const UI_ACTIONS: [&str; 18] = [
    "ui.notes",
    "ui.toggle_panels",
    "ui.new",
    "ui.open",
    "ui.save",
    "ui.save_as",
    "ui.quit",
    "ui.tool",
    "ui.paste",
    "view.zoom_in",
    "view.zoom_out",
    "view.zoom_fit",
    "ui.preview",
    // M3: import prompt and the offset / array windows.
    "ui.import",
    "ui.offset",
    "ui.grid_array",
    "ui.circular_array",
    // M5: G-code export prompt.
    "ui.export_gcode",
];

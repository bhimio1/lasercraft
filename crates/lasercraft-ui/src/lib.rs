//! LaserCraft's egui front end.
//!
//! The UI holds no business logic: it draws the project it is given and turns
//! clicks, menu picks, shortcuts and console lines into [`Action`]s. The app
//! runs those through the command session (`docs/spec/ui-shell.md`).

mod camera_panel;
mod camera_windows;
mod canvas;
pub mod colors;
mod device_dialogs;
mod dialogs;
mod dock;
mod icons;
pub mod interaction;
pub mod keymap;
mod laser_panel;
mod layers_panel;
mod materials_panel;
mod menus;
mod preview;
mod setup_windows;
mod shell;
mod test_windows;
mod textures;
pub mod theme;
mod tool_column;
pub mod view;
pub mod widgets;

pub use camera_panel::{CameraAction, CameraLive, CameraStatus, CameraView, LensCapture};
pub use dock::DockPanel;
pub use laser_panel::{
    ANCHORS, Busy, CONSOLE_LINES, ConsoleLine, Enabled, LaserAction, LaserView, MachineWord, clock,
};
pub use preview::{Preview, PreviewKind, PreviewMove};
pub use shell::{Action, Shell, ShellView};
pub use test_windows::{TestInfo, TestKind, TestRequest};

/// The `ui.*` and `view.*` names a keymap may bind besides registry
/// commands. They need something only the front end has (a path prompt, the
/// saved-file path, the pointer, the tool, the zoom).
pub const UI_ACTIONS: [&str; 37] = [
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
    // M6: the Laser panel, and laser actions that take its settings.
    "ui.laser",
    "ui.device_start",
    "ui.device_frame",
    "ui.device_frame_outline",
    "ui.device_pause",
    "ui.jog_up",
    "ui.jog_down",
    "ui.jog_left",
    "ui.jog_right",
    // M7: the Adjust Image window.
    "ui.adjust_image",
    // M8: device profile and test pattern windows.
    "ui.device_profiles",
    "ui.device_profile",
    "ui.test_speed_power",
    "ui.test_interval",
    // M9: job setup windows.
    "ui.rotary",
    "ui.taper",
    "ui.registration",
    "ui.camera_list",
    "ui.camera_capture",
];

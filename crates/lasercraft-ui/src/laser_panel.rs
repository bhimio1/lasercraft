//! The machine panels and the job bar (`docs/spec/laser.md`, "Panel
//! layout"; `docs/spec/style/machine-panels.md`): Machine (state, job
//! buttons, placement, connection), Jog (head position, pad, user origin)
//! and Console (GRBL messages). They draw [`LaserView`] data from the app
//! and turn clicks into `device.*` invocations; they hold no device logic.

use lasercraft_commands::Invocation;

use crate::Action;
use crate::keymap::Keymap;
use crate::theme::{Tokens, size, weight};
use crate::widgets::{self, Arrow, Band, Fill};

/// Job origin names, row by row from the top left (`docs/spec/laser.md`).
pub const ANCHORS: [&str; 9] = [
    "top-left",
    "top",
    "top-right",
    "left",
    "center",
    "right",
    "bottom-left",
    "bottom",
    "bottom-right",
];

/// What the reference-point dots say, in the order of [`ANCHORS`].
const ANCHOR_WORDS: [&str; 9] = [
    "top left",
    "top",
    "top right",
    "left",
    "center",
    "right",
    "bottom left",
    "bottom",
    "bottom right",
];

/// Place job at: command value, label, tooltip.
const START_FROM: [(&str, &str, &str); 3] = [
    (
        "absolute",
        "Absolute coordinates",
        "The job lands where it sits on the bed.",
    ),
    (
        "current",
        "Current position",
        "The job's reference point lands where the head is when you press Start.",
    ),
    (
        "user",
        "User origin",
        "The job's reference point lands on the user origin.",
    ),
];

/// Baud rates offered for the connection.
const BAUDS: [&str; 7] = [
    "9600", "19200", "38400", "57600", "115200", "230400", "250000",
];

/// Lines kept in the device console.
pub const CONSOLE_LINES: usize = 500;

/// What the job bar says while a constant-power (M3) job runs
/// (`docs/spec/safety.md`, rule 9).
pub const MAY_STAY_ON: &str = "The laser may stay on";

/// The tooltip of "Pause (motion only)".
const PAUSE_M3_HINT: &str = "Stops motion at once. LaserCraft then switches the laser \
    off, but it may stay on until the head has stopped. Use Stop to turn it off.";

/// A line in the device console.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleLine {
    /// Sent by us, or received.
    pub sent: bool,
    /// The text.
    pub text: String,
    /// Routine traffic, shown only with "Show all messages".
    pub routine: bool,
}

/// A stream in progress.
#[derive(Debug, Clone, PartialEq)]
pub struct Busy {
    /// "job" or "frame".
    pub kind: String,
    /// Lines accepted by the controller.
    pub acknowledged: usize,
    /// Lines in the stream.
    pub total: usize,
    /// Seconds since it started.
    pub elapsed: f64,
    /// Paused by the user.
    pub paused: bool,
    /// The program uses constant power (`M3`).
    pub constant_power: bool,
}

impl Busy {
    fn sent_percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            100.0 * self.acknowledged as f64 / self.total as f64
        }
    }
}

/// The machine state in the words the panels show
/// (`docs/spec/style/machine-panels.md`, "Machine states"). The app derives
/// it from GRBL's state and its own job state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MachineWord {
    /// No port is open.
    #[default]
    NotConnected,
    /// The port is open; the handshake has not finished.
    Connecting,
    /// Idle and ready.
    Ready,
    /// A job runs.
    Running,
    /// A frame runs.
    Framing,
    /// Motion is held.
    Paused,
    /// A jog runs.
    Jogging,
    /// The homing cycle runs.
    Homing,
    /// Motion is locked until the alarm is cleared.
    Alarm,
    /// The safety door is open.
    DoorOpen,
    /// GRBL's check mode.
    Checking,
    /// GRBL sleeps.
    Sleeping,
    /// Status reports stopped arriving.
    NoResponse,
}

impl MachineWord {
    /// The word shown.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::NotConnected => "Not connected",
            Self::Connecting => "Connecting…",
            Self::Ready => "Ready",
            Self::Running => "Running",
            Self::Framing => "Framing",
            Self::Paused => "Paused",
            Self::Jogging => "Jogging",
            Self::Homing => "Homing",
            Self::Alarm => "Alarm",
            Self::DoorOpen => "Door open",
            Self::Checking => "Checking",
            Self::Sleeping => "Sleeping",
            Self::NoResponse => "No response",
        }
    }

    fn dot(self, t: &Tokens) -> Option<egui::Color32> {
        Some(match self {
            Self::NotConnected => return None,
            Self::Ready => t.success,
            Self::Running | Self::Framing | Self::Jogging | Self::Homing => t.accent,
            Self::Paused | Self::DoorOpen | Self::NoResponse => t.warning,
            Self::Alarm => t.danger,
            Self::Connecting | Self::Checking | Self::Sleeping => t.text_muted,
        })
    }
}

/// What the app knows about the laser, for drawing. Plain data.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LaserView {
    /// The port, while connected.
    pub port: Option<String>,
    /// The baud rate, while connected.
    pub baud: Option<u32>,
    /// The handshake has finished.
    pub ready: bool,
    /// The machine state in words.
    pub word: MachineWord,
    /// Last reported head position, machine mm.
    pub position: Option<[f64; 2]>,
    /// The stored user origin, machine mm.
    pub user_origin: Option<[f64; 2]>,
    /// A stream in progress.
    pub busy: Option<Busy>,
    /// How the last job or frame ended ("Finished in 0:01:05").
    pub last_result: Option<String>,
    /// Ports to choose from.
    pub ports: Vec<String>,
    /// The device console, oldest first.
    pub console: Vec<ConsoleLine>,
    /// An alarm, shown until the machine leaves the alarm state.
    pub alarm: Option<String>,
    /// A fault to show until dismissed (a failed job, a lost connection).
    pub notice: Option<String>,
    /// Help for a refused serial port, shown until hidden.
    pub permission: Option<String>,
    /// Why Start or Frame was refused, shown until dismissed.
    pub refused: Option<String>,
}

/// App-side laser actions that are not commands: they change what the app
/// remembers, not the project or the machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaserAction {
    /// Remember the head's position as the user origin.
    SetOrigin,
    /// Forget the user origin.
    ClearOrigin,
    /// List the serial ports again.
    RefreshPorts,
    /// Hide the fault band.
    DismissNotice,
    /// Hide the serial port access help.
    DismissPermission,
    /// Hide the refused-job band.
    DismissRefused,
}

/// Which machine controls can be used now; one rule for the panels, the
/// job bar and the Machine menu.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Enabled {
    /// Pause, Resume and Stop.
    pub pause_stop: bool,
    /// Start, Frame and Frame outline.
    pub start: bool,
    /// Home.
    pub home: bool,
    /// The jog pad and Move head to.
    pub jog: bool,
    /// Go to user origin.
    pub go_origin: bool,
    /// Set user origin.
    pub set_origin: bool,
    /// Clear user origin.
    pub clear_origin: bool,
    /// Clear alarm.
    pub clear_alarm: bool,
    /// Disconnect.
    pub disconnect: bool,
    /// Port, baud rate and Refresh ports.
    pub connect: bool,
    /// The console's input line.
    pub console: bool,
    /// Place job at and the reference point.
    pub placement: bool,
}

impl Enabled {
    /// The rule (`docs/spec/laser.md`, "Enabled controls").
    #[must_use]
    pub fn of(view: &LaserView, has_output: bool) -> Self {
        use MachineWord as W;
        let connected = view.port.is_some();
        let free = view.ready && view.busy.is_none();
        let idle = free && matches!(view.word, W::Ready | W::Checking);
        Self {
            pause_stop: connected,
            start: idle && has_output,
            home: free && matches!(view.word, W::Ready | W::Alarm | W::Checking),
            jog: free && matches!(view.word, W::Ready | W::Jogging),
            go_origin: idle && view.user_origin.is_some(),
            set_origin: idle && view.position.is_some(),
            clear_origin: view.user_origin.is_some(),
            clear_alarm: free && view.word == W::Alarm,
            disconnect: connected && view.busy.is_none(),
            connect: !connected,
            console: free,
            placement: view.busy.is_none(),
        }
    }
}

/// The Pause button's label, invocation and tooltip.
#[must_use]
pub fn pause_label(view: &LaserView) -> (&'static str, &'static str, &'static str) {
    match &view.busy {
        Some(b) if b.paused => ("Resume", "device.resume", "Resume the held job"),
        Some(b) if b.constant_power => ("Pause (motion only)", "device.pause", PAUSE_M3_HINT),
        _ => ("Pause", "device.pause", "Pause the job"),
    }
}

/// The panels' own settings.
#[derive(Debug, Clone)]
pub struct LaserPanel {
    port: String,
    baud: String,
    from: usize,
    anchor: usize,
    distance: f64,
    speed: f64,
    input: String,
    show_all: bool,
    move_to: [String; 2],
    /// Set when a band asks to bring the Console to the front.
    pub(crate) show_console: bool,
    export_requested: bool,
}

impl Default for LaserPanel {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud: "115200".to_owned(),
            from: 0,
            anchor: 6,
            distance: 10.0,
            speed: 50.0,
            input: String::new(),
            show_all: false,
            move_to: [String::new(), String::new()],
            show_console: false,
            export_requested: false,
        }
    }
}

fn run(line: impl AsRef<str>) -> Option<Action> {
    Invocation::parse(line.as_ref()).ok().map(Action::Run)
}

fn fmt(v: f64) -> String {
    let text = format!("{v:.3}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// Seconds as h:mm:ss.
#[must_use]
pub fn clock(seconds: f64) -> String {
    let s = seconds.max(0.0).round() as u64;
    format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

/// "Tooltip (chord)" when `bound` has a chord.
fn hint(keymap: &Keymap, text: &str, bound: &str) -> String {
    keymap
        .chord_for(bound)
        .map_or_else(|| text.to_owned(), |c| format!("{text} ({})", c.label()))
}

impl LaserPanel {
    fn placement(&self) -> String {
        format!(
            "from={} anchor={}",
            START_FROM[self.from].0, ANCHORS[self.anchor]
        )
    }

    /// Whether Place job at is Absolute coordinates.
    pub(crate) fn absolute(&self) -> bool {
        START_FROM[self.from].0 == "absolute"
    }

    /// Starts (or frames) a test pattern with the panel's placement.
    pub(crate) fn test_job(&self, kind: &str, settings: &str, frame: bool) -> Action {
        let name = if frame {
            "device.test_frame"
        } else {
            "device.test"
        };
        let mut inv = Invocation::new(name, [kind, settings]);
        inv.args
            .extend(self.placement().split(' ').map(str::to_owned));
        if frame {
            inv.args.push(format!("speed={}", fmt(self.speed)));
        }
        Action::Run(inv)
    }

    /// Whether a port is chosen to connect to.
    #[must_use]
    pub fn can_connect(&self) -> bool {
        !self.port.trim().is_empty()
    }

    /// Connects to the chosen port at the chosen baud rate.
    #[must_use]
    pub fn connect(&self) -> Option<Action> {
        let mut inv = Invocation::parse(&format!("device.connect \"{}\"", self.port)).ok()?;
        inv.args.push(self.baud.trim().to_owned());
        Some(Action::Run(inv))
    }

    /// Sends the head to `origin` (machine mm) at the jog speed.
    #[must_use]
    pub fn go_to(&self, [x, y]: [f64; 2]) -> Option<Action> {
        run(format!(
            "device.move_to {} {} speed={}",
            fmt(x),
            fmt(y),
            fmt(self.speed)
        ))
    }

    /// The invocation for a `ui.*` laser binding, using the panels'
    /// settings; `None` for other names.
    #[must_use]
    pub fn resolve(&self, name: &str, view_paused: bool) -> Option<Option<Action>> {
        let speed = fmt(self.speed);
        Some(match name {
            "ui.device_start" => run(format!("device.start {}", self.placement())),
            "ui.device_frame" => run(format!("device.frame {} speed={speed}", self.placement())),
            "ui.device_frame_outline" => run(format!(
                "device.frame outline {} speed={speed}",
                self.placement()
            )),
            "ui.device_pause" => run(if view_paused {
                "device.resume"
            } else {
                "device.pause"
            }),
            "ui.jog_up" | "ui.jog_down" | "ui.jog_left" | "ui.jog_right" => run(format!(
                "device.jog {} {} speed={speed}",
                &name["ui.jog_".len()..],
                fmt(self.distance)
            )),
            _ => return None,
        })
    }

    /// The Machine panel. `device` draws the Device row and returns the
    /// port and baud rate of a profile chosen there.
    pub fn machine(
        &mut self,
        ui: &mut egui::Ui,
        view: &LaserView,
        keymap: &Keymap,
        has_output: bool,
        actions: &mut Vec<Action>,
        device: impl FnOnce(&mut egui::Ui) -> Option<(String, u32)>,
    ) {
        let on = Enabled::of(view, has_output);
        state_line(ui, view);
        progress(ui, view);
        ui.add_space(size::GAP_TIGHT);
        self.job_rows(ui, view, keymap, on, actions);
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("machine-body")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                self.bands(ui, view, keymap, on, actions);
                self.positioning(ui, view, keymap, on, actions);
                ui.separator();
                self.connection(ui, view, on, actions);
                if let Some((port, baud)) = device(ui) {
                    if !port.is_empty() {
                        self.port = port;
                    }
                    self.baud = baud.to_string();
                }
            });
    }

    fn job_rows(
        &mut self,
        ui: &mut egui::Ui,
        view: &LaserView,
        keymap: &Keymap,
        on: Enabled,
        actions: &mut Vec<Action>,
    ) {
        let width = ((ui.available_width() - 2.0 * size::GAP) / 3.0).max(40.0);
        let big = egui::vec2(width, size::ROW_BIG);
        let why = why_not(view, needs_output(on, view));
        ui.horizontal(|ui| {
            pause_button(ui, view, keymap, big, actions);
            stop_button(ui, view, keymap, big, actions);
            let start = ui
                .add_enabled(on.start, egui::Button::new("Start").min_size(big))
                .on_hover_text(hint(
                    keymap,
                    "Check the job against the work area, then run it",
                    "ui.device_start",
                ))
                .on_disabled_hover_text(why);
            if start.clicked() {
                start.surrender_focus();
                actions.extend(self.resolve("ui.device_start", false).flatten());
            }
        });
        ui.horizontal(|ui| {
            for (label, name, text) in [
                (
                    "Frame",
                    "ui.device_frame",
                    "Trace the job's bounding box with the laser off",
                ),
                (
                    "Frame outline",
                    "ui.device_frame_outline",
                    "Trace the tight outline around the job with the laser off",
                ),
            ] {
                let response = ui
                    .add_enabled(on.start, egui::Button::new(label).min_size(big))
                    .on_hover_text(format!(
                        "{}\nFrames at the Jog panel's Jog speed (mm/s).",
                        hint(keymap, text, name)
                    ))
                    .on_disabled_hover_text(why);
                if response.clicked() {
                    actions.extend(self.resolve(name, false).flatten());
                }
            }
            if ui
                .add(egui::Button::new("Save G-code…").min_size(big))
                .on_hover_text(hint(
                    keymap,
                    "Write the job to a G-code file",
                    "ui.export_gcode",
                ))
                .clicked()
            {
                self.export_requested = true;
            }
        });
    }

    fn bands(
        &mut self,
        ui: &mut egui::Ui,
        view: &LaserView,
        keymap: &Keymap,
        on: Enabled,
        actions: &mut Vec<Action>,
    ) {
        if view.busy.as_ref().is_some_and(|b| b.constant_power) {
            widgets::band(
                ui,
                Band::Warning,
                "Constant power (M3): the laser may stay on while paused. Stop turns it off.",
            );
        }
        if let Some(refused) = &view.refused {
            widgets::band(ui, Band::Error, refused);
            ui.horizontal(|ui| {
                if ui.button("Show in job preview").clicked() {
                    actions.push(Action::Preview);
                }
                if ui.button("Hide").clicked() {
                    actions.push(Action::Laser(LaserAction::DismissRefused));
                }
            });
        }
        if let Some(alarm) = &view.alarm {
            widgets::band(ui, Band::Error, alarm);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(on.clear_alarm, egui::Button::new("Clear alarm"))
                    .on_hover_text("Clear the alarm lock (GRBL $X) once the machine is safe")
                    .clicked()
                {
                    actions.extend(run("device.unlock"));
                }
                if ui
                    .add_enabled(on.home, egui::Button::new("Home"))
                    .on_hover_text(hint(keymap, "Run the homing cycle", "device.home"))
                    .clicked()
                {
                    actions.extend(run("device.home"));
                }
                if ui.button("Show in console").clicked() {
                    self.show_console = true;
                }
            });
        }
        if let Some(notice) = &view.notice {
            widgets::band(ui, Band::Error, notice);
            ui.horizontal(|ui| {
                if ui.button("Show in console").clicked() {
                    self.show_console = true;
                }
                if ui.button("Hide").clicked() {
                    actions.push(Action::Laser(LaserAction::DismissNotice));
                }
            });
        }
        if let Some(help) = &view.permission {
            ui.label(egui::RichText::new("Serial port access").heading());
            widgets::band(ui, Band::Warning, help);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(self.can_connect(), egui::Button::new("Try again"))
                    .clicked()
                {
                    actions.extend(self.connect());
                }
                if ui.button("Hide").clicked() {
                    actions.push(Action::Laser(LaserAction::DismissPermission));
                }
            });
        }
    }

    fn positioning(
        &mut self,
        ui: &mut egui::Ui,
        view: &LaserView,
        keymap: &Keymap,
        on: Enabled,
        actions: &mut Vec<Action>,
    ) {
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(on.home, egui::Button::new("Home"))
                .on_hover_text(hint(
                    keymap,
                    "Run the homing cycle; the laser stays off",
                    "device.home",
                ))
                .on_disabled_hover_text(why_not(view, false))
                .clicked()
            {
                actions.extend(run("device.home"));
            }
            let tip = view.user_origin.map_or_else(
                || "Move the head to the stored user origin, laser off".to_owned(),
                |[x, y]| {
                    format!(
                        "Move the head to the stored user origin, laser off\n\
                         User origin: X {x:.2} Y {y:.2} mm"
                    )
                },
            );
            let go = ui
                .add_enabled(on.go_origin, egui::Button::new("Go to user origin"))
                .on_hover_text(tip)
                .on_disabled_hover_text(if view.port.is_none() {
                    "Needs a connected machine."
                } else {
                    "Set a user origin in the Jog panel first."
                });
            if go.clicked()
                && let Some(origin) = view.user_origin
            {
                actions.extend(self.go_to(origin));
            }
        });
        ui.add_enabled_ui(on.placement, |ui| {
            egui::Grid::new("machine-placement")
                .num_columns(2)
                .spacing([size::GAP, size::GAP_TIGHT])
                .show(ui, |ui| {
                    ui.label("Place job at")
                        .on_hover_text("Where the job goes on the bed");
                    egui::ComboBox::from_id_salt("laser-start-from")
                        .selected_text(START_FROM[self.from].1)
                        .show_ui(ui, |ui| {
                            for (i, (value, label, tip)) in START_FROM.iter().enumerate() {
                                let usable = *value != "user" || view.user_origin.is_some();
                                ui.add_enabled_ui(usable, |ui| {
                                    ui.selectable_value(&mut self.from, i, *label)
                                        .on_hover_text(*tip)
                                        .on_disabled_hover_text(format!(
                                            "{tip}\nSet one in the Jog panel."
                                        ));
                                });
                            }
                        });
                    ui.end_row();
                    ui.label("Job reference point")
                        .on_hover_text("Which point of the job lands on the job origin");
                    ui.add_enabled_ui(self.from != 0, |ui| {
                        point_grid(ui, &mut self.anchor);
                    });
                    ui.end_row();
                });
        });
    }

    fn connection(
        &mut self,
        ui: &mut egui::Ui,
        view: &LaserView,
        on: Enabled,
        actions: &mut Vec<Action>,
    ) {
        if self.port.is_empty()
            && let Some(first) = view.ports.first()
        {
            self.port.clone_from(first);
        }
        ui.horizontal_wrapped(|ui| {
            if view.port.is_some() {
                if ui
                    .add_enabled(on.disconnect, egui::Button::new("Disconnect"))
                    .on_hover_text("Close the connection; Stop first if a job runs")
                    .on_disabled_hover_text("Stop the job first.")
                    .clicked()
                {
                    actions.extend(run("device.disconnect"));
                }
            } else if ui
                .add_enabled(self.can_connect(), egui::Button::new("Connect"))
                .on_hover_text("Open the serial connection to the laser")
                .clicked()
            {
                actions.extend(self.connect());
            }
            ui.add_enabled_ui(on.connect, |ui| {
                ui.label("Port");
                egui::ComboBox::from_id_salt("laser-port")
                    .selected_text(&self.port)
                    .show_ui(ui, |ui| {
                        if view.ports.is_empty() {
                            ui.add_enabled(false, egui::Button::new("No serial ports found"))
                                .on_disabled_hover_text(
                                    "Check the USB cable and that the laser is powered; \
                                     then use Refresh ports.",
                                );
                        }
                        for port in &view.ports {
                            ui.selectable_value(&mut self.port, port.clone(), port);
                        }
                    })
                    .response
                    .on_hover_text("Serial port the laser is on");
                ui.label("Baud rate");
                egui::ComboBox::from_id_salt("laser-baud")
                    .selected_text(&self.baud)
                    .show_ui(ui, |ui| {
                        for baud in BAUDS {
                            ui.selectable_value(&mut self.baud, baud.to_owned(), baud);
                        }
                    });
                if ui
                    .button("Refresh ports")
                    .on_hover_text("Look for serial ports again")
                    .clicked()
                {
                    actions.push(Action::Laser(LaserAction::RefreshPorts));
                }
            });
        });
    }

    /// The Jog panel.
    pub fn jog(
        &mut self,
        ui: &mut egui::Ui,
        view: &LaserView,
        keymap: &Keymap,
        actions: &mut Vec<Action>,
    ) {
        let on = Enabled::of(view, false);
        self.head_position(ui, view);
        self.move_head(ui, on, actions);
        ui.separator();
        ui.add_enabled_ui(on.jog, |ui| {
            let mut pressed = None;
            let pad = egui::vec2(size::ROW * 2.0, size::ROW * 1.5);
            egui::Grid::new("laser-jog")
                .spacing([size::GAP_TIGHT, size::GAP_TIGHT])
                .show(ui, |ui| {
                    let mut cell =
                        |ui: &mut egui::Ui, arrow: Option<Arrow>, name: &'static str, text| {
                            let button = match arrow {
                                Some(arrow) => widgets::arrow_button(ui, arrow, pad),
                                None => ui.add(egui::Button::new("Home").min_size(pad)),
                            };
                            if button
                                .on_hover_text(hint(keymap, text, name))
                                .on_disabled_hover_text(why_not(view, false))
                                .clicked()
                            {
                                pressed = Some(name);
                            }
                        };
                    ui.label("");
                    cell(ui, Some(Arrow::Up), "ui.jog_up", "Jog up");
                    ui.label("");
                    ui.end_row();
                    cell(ui, Some(Arrow::Left), "ui.jog_left", "Jog left");
                    cell(ui, None, "device.home", "Run the homing cycle");
                    cell(ui, Some(Arrow::Right), "ui.jog_right", "Jog right");
                    ui.end_row();
                    ui.label("");
                    cell(ui, Some(Arrow::Down), "ui.jog_down", "Jog down");
                    ui.end_row();
                });
            match pressed {
                Some("device.home") => actions.extend(run("device.home")),
                Some(name) => actions.extend(self.resolve(name, false).flatten()),
                None => {}
            }
        });
        egui::Grid::new("jog-fields")
            .num_columns(2)
            .spacing([size::GAP, size::GAP_TIGHT])
            .show(ui, |ui| {
                ui.label("Jog distance (mm)");
                ui.add(egui::DragValue::new(&mut self.distance).range(0.01..=1000.0))
                    .on_hover_text("How far one jog click moves. 0.01–1000.");
                ui.end_row();
                ui.label("Jog speed (mm/s)");
                ui.add(egui::DragValue::new(&mut self.speed).range(1.0..=1000.0))
                    .on_hover_text(
                        "Speed of jog moves, framing and Move head to. 1–1000; \
                         never above the device's maximum speed.",
                    );
                ui.end_row();
            });
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(on.set_origin, egui::Button::new("Set user origin"))
                .on_hover_text("Store the head position as the user origin")
                .on_disabled_hover_text(why_not(view, false))
                .clicked()
            {
                actions.push(Action::Laser(LaserAction::SetOrigin));
            }
            if ui
                .add_enabled(on.clear_origin, egui::Button::new("Clear user origin"))
                .on_hover_text("Forget the stored user origin")
                .on_disabled_hover_text("No user origin is set.")
                .clicked()
            {
                actions.push(Action::Laser(LaserAction::ClearOrigin));
            }
        });
    }

    fn head_position(&mut self, ui: &mut egui::Ui, view: &LaserView) {
        let t = Tokens::of(ui.ctx());
        ui.horizontal(|ui| {
            ui.label("Head position");
            let (text, color, tip) = match view.position {
                None => ("—".to_owned(), t.text_disabled, "No position reported yet"),
                Some([x, y]) => {
                    let text = format!("X {x:.2}  Y {y:.2}");
                    match view.word {
                        MachineWord::NotConnected => (text, t.text_disabled, "Last known position"),
                        MachineWord::Alarm => (
                            text,
                            t.text_muted,
                            "Position may be wrong until the machine is homed",
                        ),
                        MachineWord::NoResponse => (text, t.text_muted, "No recent report"),
                        _ => (text, t.text, "Click to copy into Move head to"),
                    }
                }
            };
            let label = egui::Label::new(egui::RichText::new(text).monospace().color(color))
                .sense(egui::Sense::click());
            if ui.add(label).on_hover_text(tip).clicked()
                && let Some([x, y]) = view.position
            {
                self.move_to = [format!("{x:.2}"), format!("{y:.2}")];
            }
        });
    }

    fn move_head(&mut self, ui: &mut egui::Ui, on: Enabled, actions: &mut Vec<Action>) {
        ui.add_enabled_ui(on.jog, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Move head to:");
                let mut enter = false;
                for (label, text) in ["X (mm)", "Y (mm)"].into_iter().zip(&mut self.move_to) {
                    ui.label(label);
                    let field = ui.add(egui::TextEdit::singleline(text).desired_width(56.0));
                    enter |= field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                }
                let go = ui
                    .button("Go")
                    .on_hover_text("Move the head to these coordinates with the laser off");
                let target = (
                    self.move_to[0].trim().parse::<f64>(),
                    self.move_to[1].trim().parse::<f64>(),
                );
                if (go.clicked() || enter)
                    && let (Ok(x), Ok(y)) = target
                {
                    actions.extend(self.go_to([x, y]));
                }
            });
        });
    }

    /// The Console panel.
    pub fn console(&mut self, ui: &mut egui::Ui, view: &LaserView, actions: &mut Vec<Action>) {
        let on = Enabled::of(view, false);
        let t = Tokens::of(ui.ctx());
        ui.horizontal(|ui| {
            let hint = if view.busy.is_some() {
                "Wait for the job to end"
            } else {
                "Type a GRBL command"
            };
            let width = (ui.available_width() - 150.0).max(80.0);
            let field = ui
                .add_enabled(
                    on.console,
                    egui::TextEdit::singleline(&mut self.input)
                        .hint_text(hint)
                        .desired_width(width),
                )
                .on_hover_text("Send a line to the controller");
            if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let line = std::mem::take(&mut self.input);
                if !line.trim().is_empty() {
                    actions.push(Action::Run(Invocation::new("device.send", [line])));
                }
                field.request_focus();
            }
            widgets::switch(ui, &mut self.show_all, "Show all messages")
                .on_hover_text("Also show status reports and every line LaserCraft sends");
        });
        egui::Frame::new()
            .fill(t.field)
            .stroke(egui::Stroke::new(weight::THIN, t.border))
            .inner_margin(size::GAP_TIGHT)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("laser-console")
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if view.port.is_none() && view.console.is_empty() {
                            let text = "Not connected. Messages from GRBL appear here \
                                        once you connect.";
                            ui.label(egui::RichText::new(text).color(t.text_muted));
                        }
                        for line in view.console.iter().filter(|l| self.show_all || !l.routine) {
                            console_line(ui, line, &t);
                        }
                    });
            });
    }

    /// Takes a pending "Save G-code…" click.
    pub(crate) fn take_export(&mut self) -> bool {
        std::mem::take(&mut self.export_requested)
    }
}

fn console_line(ui: &mut egui::Ui, line: &ConsoleLine, t: &Tokens) {
    let prefix = match (line.sent, line.routine) {
        (true, false) => "> ",
        (true, true) => "→ ",
        _ => "",
    };
    let muted = line.routine || line.text == "ok";
    let color = if muted { t.text_muted } else { t.text };
    let text = egui::RichText::new(format!("{prefix}{}", line.text))
        .monospace()
        .color(color);
    if line.text.starts_with("ALARM:") {
        egui::Frame::new().fill(t.danger_soft).show(ui, |ui| {
            ui.label(text);
        });
    } else if line.text.starts_with("error:") {
        ui.horizontal(|ui| {
            widgets::mark(ui, Band::Error);
            ui.label(text);
        });
    } else {
        ui.label(text);
    }
}

/// Whether the job buttons are blocked only by a job with no output.
fn needs_output(on: Enabled, view: &LaserView) -> bool {
    !on.start && view.ready && view.busy.is_none() && view.word == MachineWord::Ready
}

/// The second tooltip line of a disabled machine control.
pub(crate) fn why_not(view: &LaserView, no_output: bool) -> &'static str {
    if view.port.is_none() {
        "Needs a connected machine."
    } else if no_output {
        "Nothing to cut: no output layer has shapes."
    } else if view.busy.is_some() {
        "Wait for the job to end."
    } else if view.word == MachineWord::Alarm {
        "Clear the alarm first."
    } else {
        "Needs a ready machine."
    }
}

/// The state dot and word, then where the machine is connected.
fn state_line(ui: &mut egui::Ui, view: &LaserView) {
    let t = Tokens::of(ui.ctx());
    let fill = if view.word == MachineWord::Alarm {
        t.danger_soft
    } else {
        egui::Color32::TRANSPARENT
    };
    egui::Frame::new().fill(fill).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            state_word(ui, view.word, &t);
            if let Some(port) = &view.port {
                let at = view
                    .baud
                    .map(|b| format!(" at {b} baud"))
                    .unwrap_or_default();
                ui.label(egui::RichText::new(format!("on {port}{at}")).color(t.text_muted));
            }
        });
    });
}

fn state_word(ui: &mut egui::Ui, word: MachineWord, t: &Tokens) -> egui::Response {
    let color = if word == MachineWord::NotConnected {
        t.text_muted
    } else {
        t.text
    };
    let label = || egui::Label::new(egui::RichText::new(word.label()).color(color));
    // The dot always reads before the word, also in right-to-left rows.
    let rtl = ui.layout().prefer_right_to_left();
    ui.horizontal(|ui| {
        let first = rtl.then(|| ui.add(label().sense(egui::Sense::click())));
        if let Some(dot) = word.dot(t) {
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(size::ICON_SMALL, size::ROW),
                egui::Sense::hover(),
            );
            ui.painter()
                .circle_filled(rect.center(), size::ICON_SMALL / 3.0, dot);
        }
        first.unwrap_or_else(|| ui.add(label().sense(egui::Sense::click())))
    })
    .inner
}

fn progress(ui: &mut egui::Ui, view: &LaserView) {
    let t = Tokens::of(ui.ctx());
    if let Some(busy) = &view.busy {
        let percent = busy.sent_percent();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(clock(busy.elapsed)).monospace());
            let fill = if busy.paused { t.warning } else { t.accent };
            ui.add(
                egui::ProgressBar::new((percent / 100.0) as f32)
                    .fill(fill)
                    .desired_width((ui.available_width() - 70.0).max(40.0)),
            );
            ui.label(
                egui::RichText::new(format!("{percent:.0}% sent"))
                    .small()
                    .color(t.text_muted),
            )
            .on_hover_text(
                "Share of the job sent to the controller; the machine may still be working on it",
            );
        });
    } else if let Some(result) = &view.last_result {
        ui.label(egui::RichText::new(result).color(t.text_muted));
    }
}

pub(crate) fn pause_button(
    ui: &mut egui::Ui,
    view: &LaserView,
    keymap: &Keymap,
    min: egui::Vec2,
    actions: &mut Vec<Action>,
) {
    let (label, line, tip) = pause_label(view);
    let response = ui
        .add_enabled(view.port.is_some(), egui::Button::new(label).min_size(min))
        .on_hover_text(hint(keymap, tip, "ui.device_pause"))
        .on_disabled_hover_text("Needs a connected machine.");
    if response.clicked() {
        response.surrender_focus();
        actions.extend(run(line));
    }
}

pub(crate) fn stop_button(
    ui: &mut egui::Ui,
    view: &LaserView,
    keymap: &Keymap,
    min: egui::Vec2,
    actions: &mut Vec<Action>,
) {
    let tip = format!(
        "{}\nStreaming ends; the job cannot be resumed.",
        hint(
            keymap,
            "Stop at once and reset the controller",
            "device.stop"
        )
    );
    let response = ui
        .add_enabled_ui(view.port.is_some(), |ui| {
            widgets::filled(ui, "Stop", Fill::Danger, min)
        })
        .inner
        .on_hover_text(tip)
        .on_disabled_hover_text("Needs a connected machine.");
    if view.busy.as_ref().is_some_and(|b| b.constant_power) {
        let t = Tokens::of(ui.ctx());
        ui.painter().rect_stroke(
            response.rect.expand(2.0),
            crate::theme::ROUNDED,
            egui::Stroke::new(weight::HEAVY, t.accent),
            egui::StrokeKind::Outside,
        );
    }
    if response.clicked() {
        response.surrender_focus();
        actions.extend(run("device.stop"));
    }
}

/// The job bar: machine state, sent share, the constant-power note, Pause
/// and Stop, right-aligned (`docs/spec/safety.md`, rule 7). Returns whether
/// the state word was clicked.
pub fn job_bar(
    ui: &mut egui::Ui,
    view: &LaserView,
    keymap: &Keymap,
    actions: &mut Vec<Action>,
) -> bool {
    let t = Tokens::of(ui.ctx());
    let m3 = view
        .busy
        .as_ref()
        .is_some_and(|b| b.constant_power && !b.paused);
    let fill = if view.word == MachineWord::Alarm {
        t.danger_soft
    } else if m3 {
        t.warning_soft
    } else {
        t.window
    };
    ui.painter().rect_filled(ui.max_rect(), 0.0, fill);
    let mut clicked = false;
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        stop_button(ui, view, keymap, egui::vec2(72.0, size::ROW), actions);
        pause_button(ui, view, keymap, egui::vec2(64.0, size::ROW), actions);
        if m3 {
            ui.label(egui::RichText::new(MAY_STAY_ON).small().color(t.text))
                .on_hover_text(
                    "This job uses constant power (M3): Pause stops motion only. \
                     Use Stop to turn the laser off.",
                );
            widgets::mark(ui, Band::Warning);
        }
        if let Some(busy) = &view.busy {
            ui.label(
                egui::RichText::new(format!("{:.0}% sent", busy.sent_percent()))
                    .small()
                    .color(t.text_muted),
            )
            .on_hover_text(
                "Share of the job sent to the controller; the machine may still be working on it",
            );
        }
        clicked = state_word(ui, view.word, &t)
            .on_hover_text("Show the Machine panel")
            .clicked();
    });
    clicked
}

/// The 3×3 job reference point grid: nine dots on a square.
fn point_grid(ui: &mut egui::Ui, chosen: &mut usize) {
    let t = Tokens::of(ui.ctx());
    let cell = size::ROW;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(cell * 3.0, cell * 3.0), egui::Sense::hover());
    let enabled = ui.is_enabled();
    let center =
        |i: usize| rect.min + egui::vec2((i % 3) as f32 + 0.5, (i / 3) as f32 + 0.5) * cell;
    let line = egui::Stroke::new(weight::HAIRLINE, t.border);
    ui.painter()
        .rect_filled(rect, crate::theme::ROUNDED, t.field);
    for k in 0..3 {
        ui.painter()
            .line_segment([center(k * 3), center(k * 3 + 2)], line);
        ui.painter().line_segment([center(k), center(k + 6)], line);
    }
    for (i, word) in ANCHOR_WORDS.iter().enumerate() {
        let dot = egui::Rect::from_center_size(center(i), egui::vec2(cell, cell));
        let response = ui
            .interact(
                dot,
                ui.make_persistent_id(("ref-point", i)),
                egui::Sense::click(),
            )
            .on_hover_text(format!("Job reference point: {word}"));
        if response.clicked() {
            *chosen = i;
        }
        if response.hovered() && enabled {
            ui.painter().circle_filled(center(i), cell * 0.4, t.hover);
        }
        let (r, color) = match (i == *chosen, enabled) {
            (true, true) => (cell * 0.28, t.accent),
            (true, false) => (cell * 0.28, t.text_disabled),
            (false, true) => (cell * 0.18, t.text_muted),
            (false, false) => (cell * 0.18, t.text_disabled),
        };
        ui.painter().circle_filled(center(i), r, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connected(word: MachineWord) -> LaserView {
        LaserView {
            port: Some("simulator".into()),
            ready: true,
            word,
            position: Some([1.0, 2.0]),
            ..LaserView::default()
        }
    }

    #[test]
    fn enabled_controls_follow_the_machine_state() {
        let off = Enabled::of(&LaserView::default(), true);
        assert!(!off.pause_stop && !off.start && off.connect && !off.home);
        let ready = Enabled::of(&connected(MachineWord::Ready), true);
        assert!(ready.pause_stop && ready.start && ready.home && ready.jog);
        assert!(!ready.clear_alarm);
        assert!(!Enabled::of(&connected(MachineWord::Ready), false).start);
        let alarm = Enabled::of(&connected(MachineWord::Alarm), true);
        assert!(alarm.pause_stop && !alarm.start && alarm.home && alarm.clear_alarm);
        assert!(!alarm.jog);
        let mut running = connected(MachineWord::Running);
        running.busy = Some(Busy {
            kind: "job".into(),
            acknowledged: 0,
            total: 1,
            elapsed: 0.0,
            paused: false,
            constant_power: true,
        });
        let on = Enabled::of(&running, true);
        assert!(on.pause_stop && !on.start && !on.disconnect && !on.console);
        assert!(!on.placement);
        assert_eq!(pause_label(&running).0, "Pause (motion only)");
        let mut connecting = connected(MachineWord::Connecting);
        connecting.ready = false;
        assert!(Enabled::of(&connecting, true).pause_stop);
    }

    #[test]
    fn clock_reads_hours_minutes_seconds() {
        assert_eq!(clock(0.0), "0:00:00");
        assert_eq!(clock(65.4), "0:01:05");
        assert_eq!(clock(3725.0), "1:02:05");
    }
}

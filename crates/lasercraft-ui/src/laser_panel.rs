//! The Laser panel, its status-bar controls and the alarm and notice windows
//! (`docs/spec/laser.md`). It draws [`LaserView`] data from the app and
//! turns clicks into `device.*` invocations; it holds no device logic.

use lasercraft_commands::Invocation;

use crate::Action;

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

/// Start From modes: command value, label.
const START_FROM: [(&str, &str); 3] = [
    ("absolute", "Absolute coordinates"),
    ("current", "Current position"),
    ("user", "User origin"),
];

/// Lines kept in the device console.
pub const CONSOLE_LINES: usize = 500;

/// A line in the device console.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleLine {
    /// Sent by us, or received.
    pub sent: bool,
    /// The text.
    pub text: String,
    /// Routine traffic, shown only with "Show all".
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

/// What the app knows about the laser, for drawing. Plain data.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LaserView {
    /// The port, while connected.
    pub port: Option<String>,
    /// The handshake has finished.
    pub ready: bool,
    /// Machine state text (`Idle`, `Run`, `Alarm`, …).
    pub state: String,
    /// Last reported head position, machine mm.
    pub position: Option<[f64; 2]>,
    /// The stored user origin, machine mm.
    pub user_origin: Option<[f64; 2]>,
    /// A stream in progress.
    pub busy: Option<Busy>,
    /// Ports to choose from.
    pub ports: Vec<String>,
    /// The device console, oldest first.
    pub console: Vec<ConsoleLine>,
    /// An alarm to show until dismissed.
    pub alarm: Option<String>,
    /// A message to show until dismissed (permission help, a failed job).
    pub notice: Option<String>,
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
    /// Close the alarm window.
    DismissAlarm,
    /// Close the notice window.
    DismissNotice,
}

/// The panel's own settings.
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

impl LaserPanel {
    fn placement(&self) -> String {
        format!(
            "from={} anchor={}",
            START_FROM[self.from].0, ANCHORS[self.anchor]
        )
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

    /// The invocation for a `ui.*` laser binding, using the panel's
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

    /// Draws the panel. Returns what the user asked for.
    pub fn show(&mut self, ui: &mut egui::Ui, view: &LaserView, actions: &mut Vec<Action>) {
        self.connection(ui, view, actions);
        ui.separator();
        let ready = view.ready;
        let idle = ready && view.busy.is_none();
        ui.horizontal_wrapped(|ui| {
            for (label, name, hint) in [
                ("Start", "ui.device_start", "Run the job (Alt+S)"),
                ("Frame", "ui.device_frame", "Trace the job's box, laser off"),
                (
                    "Frame outline",
                    "ui.device_frame_outline",
                    "Trace the job's outline, laser off",
                ),
            ] {
                if ui
                    .add_enabled(idle, egui::Button::new(label))
                    .on_hover_text(hint)
                    .clicked()
                {
                    actions.extend(self.resolve(name, false).flatten());
                }
            }
        });
        stop_and_pause(ui, view, actions);
        progress(ui, view);
        ui.separator();
        self.placement_controls(ui);
        ui.separator();
        self.move_controls(ui, view, idle, actions);
        ui.separator();
        self.console(ui, view, actions);
    }

    fn connection(&mut self, ui: &mut egui::Ui, view: &LaserView, actions: &mut Vec<Action>) {
        if self.port.is_empty()
            && let Some(first) = view.ports.first()
        {
            self.port.clone_from(first);
        }
        ui.horizontal_wrapped(|ui| {
            ui.add_enabled_ui(view.port.is_none(), |ui| {
                egui::ComboBox::from_id_salt("laser-port")
                    .selected_text(if self.port.is_empty() {
                        "No ports"
                    } else {
                        &self.port
                    })
                    .show_ui(ui, |ui| {
                        for port in &view.ports {
                            ui.selectable_value(&mut self.port, port.clone(), port);
                        }
                    });
                ui.add(egui::TextEdit::singleline(&mut self.baud).desired_width(60.0))
                    .on_hover_text("Baud rate");
                if ui.button("Refresh").clicked() {
                    actions.push(Action::Laser(LaserAction::RefreshPorts));
                }
            });
            if view.port.is_some() {
                if ui.button("Disconnect").clicked() {
                    actions.extend(run("device.disconnect"));
                }
            } else if ui
                .add_enabled(self.can_connect(), egui::Button::new("Connect"))
                .clicked()
            {
                actions.extend(self.connect());
            }
        });
        ui.label(status_text(view));
    }

    fn placement_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Start from");
            egui::ComboBox::from_id_salt("laser-start-from")
                .selected_text(START_FROM[self.from].1)
                .show_ui(ui, |ui| {
                    for (i, (_, label)) in START_FROM.iter().enumerate() {
                        ui.selectable_value(&mut self.from, i, *label);
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label("Job origin");
            ui.add_enabled_ui(self.from != 0, |ui| {
                egui::Grid::new("laser-anchor")
                    .spacing([2.0, 2.0])
                    .show(ui, |ui| {
                        for (i, name) in ANCHORS.iter().enumerate() {
                            let dot = egui::Button::new(" ")
                                .selected(self.anchor == i)
                                .min_size(egui::vec2(18.0, 18.0));
                            if ui.add(dot).on_hover_text(*name).clicked() {
                                self.anchor = i;
                            }
                            if i % 3 == 2 {
                                ui.end_row();
                            }
                        }
                    });
            });
        });
    }

    fn move_controls(
        &mut self,
        ui: &mut egui::Ui,
        view: &LaserView,
        idle: bool,
        actions: &mut Vec<Action>,
    ) {
        ui.label("Move");
        ui.add_enabled_ui(idle, |ui| {
            let mut pressed = None;
            egui::Grid::new("laser-jog").show(ui, |ui| {
                let mut jog = |ui: &mut egui::Ui, label: &str, name: &'static str| {
                    if ui.button(label).on_hover_text(name).clicked() {
                        pressed = Some(name);
                    }
                };
                ui.label("");
                jog(ui, "Up", "ui.jog_up");
                ui.label("");
                ui.end_row();
                jog(ui, "Left", "ui.jog_left");
                jog(ui, "Home", "device.home");
                jog(ui, "Right", "ui.jog_right");
                ui.end_row();
                ui.label("");
                jog(ui, "Down", "ui.jog_down");
                ui.end_row();
            });
            match pressed {
                Some("device.home") => actions.extend(run("device.home")),
                Some(name) => actions.extend(self.resolve(name, false).flatten()),
                None => {}
            }
            ui.horizontal(|ui| {
                ui.label("Distance");
                ui.add(
                    egui::DragValue::new(&mut self.distance)
                        .range(0.01..=1000.0)
                        .suffix(" mm"),
                );
                ui.label("Speed");
                ui.add(
                    egui::DragValue::new(&mut self.speed)
                        .range(1.0..=1000.0)
                        .suffix(" mm/s"),
                );
            });
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(view.position.is_some(), egui::Button::new("Set origin"))
                    .clicked()
                {
                    actions.push(Action::Laser(LaserAction::SetOrigin));
                }
                if ui
                    .add_enabled(
                        view.user_origin.is_some(),
                        egui::Button::new("Clear origin"),
                    )
                    .clicked()
                {
                    actions.push(Action::Laser(LaserAction::ClearOrigin));
                }
                if let Some(origin) = view.user_origin
                    && ui.button("Go to origin").clicked()
                {
                    actions.extend(self.go_to(origin));
                }
            });
        });
        if view.ready
            && view.busy.is_none()
            && ui
                .button("Unlock")
                .on_hover_text("Clear an alarm ($X); the position may be wrong")
                .clicked()
        {
            actions.extend(run("device.unlock"));
        }
        if let Some([x, y]) = view.user_origin {
            ui.small(format!("User origin X{} Y{}", fmt(x), fmt(y)));
        }
    }

    fn console(&mut self, ui: &mut egui::Ui, view: &LaserView, actions: &mut Vec<Action>) {
        ui.horizontal(|ui| {
            ui.label("Console");
            ui.checkbox(&mut self.show_all, "Show all");
        });
        let field = ui.add(
            egui::TextEdit::singleline(&mut self.input)
                .hint_text("Send to the controller, e.g. $$")
                .desired_width(f32::INFINITY),
        );
        if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let line = std::mem::take(&mut self.input);
            if !line.trim().is_empty() {
                actions.push(Action::Run(Invocation::new("device.send", [line])));
            }
            field.request_focus();
        }
        egui::ScrollArea::vertical()
            .id_salt("laser-console")
            .stick_to_bottom(true)
            .max_height(160.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for line in view.console.iter().filter(|l| self.show_all || !l.routine) {
                    let prefix = if line.sent { "> " } else { "" };
                    ui.monospace(format!("{prefix}{}", line.text));
                }
            });
    }
}

fn status_text(view: &LaserView) -> String {
    match (&view.port, view.ready) {
        (None, _) => "Disconnected".to_owned(),
        (Some(port), false) => format!("Connecting to {port}…"),
        (Some(port), true) => {
            let at = view
                .position
                .map(|[x, y]| format!(" at X{} Y{}", fmt(x), fmt(y)))
                .unwrap_or_default();
            let what = if view.busy.is_some() { "Busy" } else { "Ready" };
            format!("{what} ({}){at} on {port}", view.state)
        }
    }
}

fn progress(ui: &mut egui::Ui, view: &LaserView) {
    if let Some(busy) = &view.busy {
        let fraction = if busy.total > 0 {
            busy.acknowledged as f32 / busy.total as f32
        } else {
            0.0
        };
        ui.add(egui::ProgressBar::new(fraction).text(format!(
            "{} {}/{} lines, {:.0} s{}",
            busy.kind,
            busy.acknowledged,
            busy.total,
            busy.elapsed,
            if busy.paused { ", paused" } else { "" }
        )));
    }
}

/// Pause/Resume and Stop: shown whenever a laser is connected (safety rule 7).
pub fn stop_and_pause(ui: &mut egui::Ui, view: &LaserView, actions: &mut Vec<Action>) {
    if view.port.is_none() {
        return;
    }
    ui.horizontal(|ui| {
        let paused = view.busy.as_ref().is_some_and(|b| b.paused);
        let constant = view.busy.as_ref().is_some_and(|b| b.constant_power);
        let (label, line) = match (paused, constant) {
            (true, _) => ("Resume", "device.resume"),
            (false, true) => ("Pause (motion)", "device.pause"),
            (false, false) => ("Pause", "device.pause"),
        };
        let pause = ui.button(label).on_hover_text(if constant {
            "Feed hold; the laser is switched off once the head has stopped"
        } else {
            "Feed hold (Pause key)"
        });
        if pause.clicked() {
            actions.extend(run(line));
        }
        let stop = egui::Button::new(egui::RichText::new("Stop").strong())
            .fill(egui::Color32::from_rgb(170, 40, 40));
        if ui
            .add(stop)
            .on_hover_text("Stop the job (Ctrl+Pause)")
            .clicked()
        {
            actions.extend(run("device.stop"));
        }
    });
}

/// The alarm and notice windows.
pub fn windows(ctx: &egui::Context, view: &LaserView, actions: &mut Vec<Action>) {
    if let Some(alarm) = &view.alarm {
        egui::Window::new("Laser alarm")
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(alarm);
                ui.label("Motion is locked until the alarm is cleared.");
                ui.horizontal(|ui| {
                    if ui
                        .button("Unlock")
                        .on_hover_text("$X: the position may be wrong")
                        .clicked()
                    {
                        actions.extend(run("device.unlock"));
                        actions.push(Action::Laser(LaserAction::DismissAlarm));
                    }
                    if ui.button("Home").on_hover_text("$H").clicked() {
                        actions.extend(run("device.home"));
                        actions.push(Action::Laser(LaserAction::DismissAlarm));
                    }
                    if ui.button("Close").clicked() {
                        actions.push(Action::Laser(LaserAction::DismissAlarm));
                    }
                });
            });
    }
    if let Some(notice) = &view.notice {
        egui::Window::new("Laser")
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(notice);
                if ui.button("OK").clicked() {
                    actions.push(Action::Laser(LaserAction::DismissNotice));
                }
            });
    }
}

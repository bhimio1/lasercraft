//! The app's side of the laser (`docs/spec/laser.md`): owns the device
//! thread's handle, turns its events into [`LaserView`] data for the shell,
//! and remembers the user origin.

use std::time::Instant;

use eframe::egui;
use lasercraft_commands::DeviceContext;
use lasercraft_device::{
    DeviceHandle, Event, MachineState, Outcome, Request, StreamKind, list_ports, spawn,
    uses_constant_power,
};
use lasercraft_geom::Point;
use lasercraft_ui::{Busy, CONSOLE_LINES, ConsoleLine, LaserAction, LaserView, MachineWord, clock};

/// The laser connection as the app sees it.
pub struct Laser {
    handle: DeviceHandle,
    view: LaserView,
    s_max: Option<f64>,
    started: Option<Instant>,
    /// GRBL's last reported state.
    state: Option<MachineState>,
    /// Status reports stopped arriving; cleared by the next one.
    stalled: bool,
}

/// The machine state in words (`docs/spec/laser.md`, "Machine states"):
/// from the connection, GRBL's state and the stream LaserCraft runs.
pub fn machine_word(view: &LaserView, state: Option<MachineState>, stalled: bool) -> MachineWord {
    use MachineState as S;
    if view.port.is_none() {
        return MachineWord::NotConnected;
    }
    if !view.ready {
        return MachineWord::Connecting;
    }
    if stalled {
        return MachineWord::NoResponse;
    }
    let framing = view.busy.as_ref().is_some_and(|b| b.kind == "frame");
    match state {
        Some(S::Alarm) => MachineWord::Alarm,
        Some(S::Hold) => MachineWord::Paused,
        Some(S::Door) => MachineWord::DoorOpen,
        _ if framing => MachineWord::Framing,
        Some(S::Run) => MachineWord::Running,
        _ if view.busy.is_some() => MachineWord::Running,
        Some(S::Jog) => MachineWord::Jogging,
        Some(S::Home) => MachineWord::Homing,
        Some(S::Check) => MachineWord::Checking,
        Some(S::Sleep) => MachineWord::Sleeping,
        Some(S::Idle) | None => MachineWord::Ready,
    }
}

impl Laser {
    /// Starts the device thread; it asks `ctx` for a repaint when events
    /// arrive.
    pub fn new(ctx: &egui::Context) -> Self {
        let ctx = ctx.clone();
        Self {
            handle: spawn(move || ctx.request_repaint()),
            view: LaserView {
                ports: list_ports(),
                ..LaserView::default()
            },
            s_max: None,
            started: None,
            state: None,
            stalled: false,
        }
    }

    /// What the shell draws.
    pub fn view(&self) -> &LaserView {
        &self.view
    }

    /// What `device.*` commands may need filled in.
    pub fn context(&self) -> DeviceContext {
        let point = |p: Option<[f64; 2]>| p.map(|[x, y]| Point::new(x, y));
        DeviceContext {
            head: point(self.view.position),
            user_origin: point(self.view.user_origin),
            s_max: self.s_max,
        }
    }

    /// Hands a command's request to the device thread.
    pub fn send(&mut self, request: Request) {
        match &request {
            Request::Connect { port, baud } => {
                self.view.port = Some(port.clone());
                self.view.baud = Some(*baud);
                self.view.ready = false;
                self.view.permission = None;
                self.state = None;
                self.stalled = false;
            }
            Request::Stream { kind, gcode } => {
                // Shown at once; the controller may still refuse it.
                self.view.busy = Some(Busy {
                    kind: kind.to_string(),
                    acknowledged: 0,
                    total: 0,
                    elapsed: 0.0,
                    paused: false,
                    constant_power: *kind == StreamKind::Job && uses_constant_power(gcode),
                });
                self.started = Some(Instant::now());
                self.view.last_result = None;
                self.view.refused = None;
            }
            Request::Pause => {
                if let Some(busy) = &mut self.view.busy {
                    busy.paused = true;
                }
            }
            Request::Resume => {
                if let Some(busy) = &mut self.view.busy {
                    busy.paused = false;
                }
            }
            _ => {}
        }
        if !self.handle.send(request) {
            self.view.notice =
                Some("The connection to the machine stopped working. Restart LaserCraft.".into());
        }
        self.update_word();
    }

    /// Carries out an app-side laser action.
    pub fn act(&mut self, action: &LaserAction) -> Option<String> {
        match action {
            LaserAction::SetOrigin => {
                self.view.user_origin = self.view.position;
                self.view
                    .position
                    .map(|[x, y]| format!("User origin set to X {x:.2} Y {y:.2} mm."))
            }
            LaserAction::ClearOrigin => {
                self.view.user_origin = None;
                Some("User origin cleared.".to_owned())
            }
            LaserAction::RefreshPorts => {
                self.view.ports = list_ports();
                None
            }
            LaserAction::DismissNotice => {
                self.view.notice = None;
                None
            }
            LaserAction::DismissPermission => {
                self.view.permission = None;
                None
            }
            LaserAction::DismissRefused => {
                self.view.refused = None;
                None
            }
        }
    }

    /// Shows why Start or Frame was refused.
    pub fn refuse(&mut self, reason: String) {
        self.view.refused = Some(reason);
    }

    fn update_word(&mut self) {
        self.view.word = machine_word(&self.view, self.state, self.stalled);
    }

    fn console(&mut self, sent: bool, text: String, routine: bool) {
        self.view.console.push(ConsoleLine {
            sent,
            text,
            routine,
        });
        if self.view.console.len() > CONSOLE_LINES {
            let excess = self.view.console.len() - CONSOLE_LINES;
            self.view.console.drain(..excess);
        }
    }

    /// Applies the device thread's events; returns lines for the app log.
    pub fn poll(&mut self) -> Vec<String> {
        let mut log = Vec::new();
        for event in self.handle.events() {
            if let Some(line) = self.apply(event) {
                log.push(line);
            }
        }
        if let (Some(busy), Some(started)) = (&mut self.view.busy, self.started) {
            busy.elapsed = started.elapsed().as_secs_f64();
        }
        self.update_word();
        log
    }

    fn apply(&mut self, event: Event) -> Option<String> {
        match event {
            Event::Connected { port } => {
                self.view.port = Some(port.clone());
                return Some(format!("Connected to {port}."));
            }
            Event::Ready(info) => {
                self.view.ready = true;
                self.s_max = info.s_max;
                let version = info.version.as_deref().unwrap_or("(version unknown)");
                let s_max = info.s_max.map_or("unknown".to_owned(), |s| s.to_string());
                return Some(match info.laser_mode {
                    Some(false) => format!(
                        "GRBL {version} ready.\nWarning: laser mode is off ($32=0), so jobs \
                         are refused. Turn it on in GRBL settings ($32=1)."
                    ),
                    Some(true) => format!(
                        "GRBL {version} ready. Laser mode on; full-power S value ($30) {s_max}."
                    ),
                    None => format!(
                        "GRBL {version} ready. Laser mode unknown; full-power S value ($30) \
                         {s_max}."
                    ),
                });
            }
            Event::Disconnected { reason } => {
                let port = self.view.port.take().unwrap_or_default();
                let during_job = self.view.busy.is_some();
                self.view.ready = false;
                self.view.busy = None;
                self.view.baud = None;
                self.view.alarm = None;
                self.state = None;
                self.stalled = false;
                if during_job {
                    let text = format!(
                        "The connection to {port} was lost during the job. Streaming \
                         stopped. Check the machine before connecting again."
                    );
                    self.view.notice = Some(text.clone());
                    return Some(text);
                }
                return Some(format!("Disconnected from {port} ({reason})."));
            }
            Event::PermissionDenied { port, help } => {
                self.view.port = None;
                self.view.baud = None;
                self.view.permission = Some(help);
                return Some(format!("Cannot connect to {port}: permission denied."));
            }
            Event::Status { state, position } => {
                self.state = Some(state);
                self.stalled = false;
                if state != MachineState::Alarm {
                    self.view.alarm = None;
                }
                if let Some(p) = position {
                    self.view.position = Some([p[0], p[1]]);
                }
            }
            Event::Alarm { code, message } => {
                self.state = Some(MachineState::Alarm);
                let text = match code {
                    Some(code) => format!("Alarm {code}: {}", message.trim_end_matches('.')),
                    None => format!("Alarm: {}", message.trim_end_matches('.')),
                };
                let text = format!(
                    "{text}. Streaming stopped. Check the machine, home it if the \
                     position may be lost, then click Clear alarm."
                );
                self.view.alarm = Some(text.clone());
                return Some(text);
            }
            Event::Stalled => {
                self.stalled = true;
                let text = "No response from the controller. Streaming stopped. Check the \
                            cable and power, then connect again.";
                self.view.notice = Some(text.to_owned());
                return Some(text.to_owned());
            }
            Event::Console {
                sent,
                text,
                routine,
            } => self.console(sent, text, routine),
            Event::Progress {
                acknowledged,
                total,
                ..
            } => {
                if let Some(busy) = &mut self.view.busy {
                    busy.acknowledged = acknowledged;
                    busy.total = total;
                }
            }
            Event::Finished { kind, outcome } => {
                let busy = self.view.busy.take();
                self.started = None;
                let what = if kind == StreamKind::Frame {
                    "Framing"
                } else {
                    "Job"
                };
                let sent = busy.as_ref().map_or(0.0, |b| {
                    if b.total == 0 {
                        0.0
                    } else {
                        100.0 * b.acknowledged as f64 / b.total as f64
                    }
                });
                let elapsed = busy.as_ref().map_or(0.0, |b| b.elapsed);
                let text = match &outcome {
                    Outcome::Completed => format!("{what} finished in {}.", clock(elapsed)),
                    Outcome::Stopped => format!("{what} stopped at {sent:.0}% sent."),
                    Outcome::Failed(reason) => {
                        self.view.notice = Some(format!(
                            "The {} stopped: {reason}. Streaming stopped; feed hold and \
                             reset were sent.",
                            what.to_lowercase()
                        ));
                        format!("{what} stopped by a fault at {sent:.0}% sent.")
                    }
                };
                self.view.last_result = Some(text.clone());
                return Some(text);
            }
            Event::Rejected(reason) => {
                if self.view.busy.as_ref().is_some_and(|b| b.total == 0) {
                    // The stream we showed as started was refused.
                    self.view.busy = None;
                    self.started = None;
                }
                let text = format!("Cannot run: {reason}");
                self.view.refused = Some(text.clone());
                return Some(text);
            }
        }
        None
    }

    /// Whether the UI should keep repainting to show live status.
    pub fn connected(&self) -> bool {
        self.view.port.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_words_follow_connection_state_and_stream() {
        let mut view = LaserView::default();
        assert_eq!(machine_word(&view, None, false), MachineWord::NotConnected);
        view.port = Some("simulator".into());
        assert_eq!(machine_word(&view, None, false), MachineWord::Connecting);
        view.ready = true;
        let word = |view: &LaserView, s| machine_word(view, Some(s), false);
        assert_eq!(word(&view, MachineState::Idle), MachineWord::Ready);
        assert_eq!(word(&view, MachineState::Alarm), MachineWord::Alarm);
        assert_eq!(word(&view, MachineState::Hold), MachineWord::Paused);
        assert_eq!(word(&view, MachineState::Door), MachineWord::DoorOpen);
        assert_eq!(word(&view, MachineState::Jog), MachineWord::Jogging);
        assert_eq!(word(&view, MachineState::Home), MachineWord::Homing);
        assert_eq!(word(&view, MachineState::Check), MachineWord::Checking);
        assert_eq!(word(&view, MachineState::Sleep), MachineWord::Sleeping);
        assert_eq!(
            machine_word(&view, Some(MachineState::Idle), true),
            MachineWord::NoResponse
        );
        view.busy = Some(Busy {
            kind: "frame".into(),
            acknowledged: 0,
            total: 1,
            elapsed: 0.0,
            paused: false,
            constant_power: false,
        });
        assert_eq!(word(&view, MachineState::Run), MachineWord::Framing);
        assert_eq!(word(&view, MachineState::Hold), MachineWord::Paused);
        view.busy.as_mut().unwrap().kind = "job".into();
        assert_eq!(word(&view, MachineState::Idle), MachineWord::Running);
    }
}

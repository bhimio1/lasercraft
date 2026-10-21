//! The app's side of the laser (`docs/spec/laser.md`): owns the device
//! thread's handle, turns its events into [`LaserView`] data for the shell,
//! and remembers the user origin.

use std::time::Instant;

use eframe::egui;
use lasercraft_commands::DeviceContext;
use lasercraft_device::{
    DeviceHandle, Event, Outcome, Request, StreamKind, list_ports, spawn, uses_constant_power,
};
use lasercraft_geom::Point;
use lasercraft_ui::{Busy, CONSOLE_LINES, ConsoleLine, LaserAction, LaserView};

/// The laser connection as the app sees it.
pub struct Laser {
    handle: DeviceHandle,
    view: LaserView,
    s_max: Option<f64>,
    started: Option<Instant>,
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
            Request::Connect { port, .. } => {
                self.view.port = Some(port.clone());
                self.view.ready = false;
                self.view.state = "connecting".to_owned();
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
            self.view.notice = Some("The device thread has stopped; restart LaserCraft.".into());
        }
    }

    /// Carries out an app-side laser action.
    pub fn act(&mut self, action: &LaserAction) -> Option<String> {
        match action {
            LaserAction::SetOrigin => {
                self.view.user_origin = self.view.position;
                self.view
                    .position
                    .map(|[x, y]| format!("user origin set to X{x:.3} Y{y:.3}"))
            }
            LaserAction::ClearOrigin => {
                self.view.user_origin = None;
                Some("user origin cleared".to_owned())
            }
            LaserAction::RefreshPorts => {
                self.view.ports = list_ports();
                None
            }
            LaserAction::DismissAlarm => {
                self.view.alarm = None;
                None
            }
            LaserAction::DismissNotice => {
                self.view.notice = None;
                None
            }
        }
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
        log
    }

    fn apply(&mut self, event: Event) -> Option<String> {
        match event {
            Event::Connected { port } => {
                self.view.port = Some(port.clone());
                return Some(format!("laser: connected to {port}"));
            }
            Event::Ready(info) => {
                self.view.ready = true;
                self.s_max = info.s_max;
                let laser_mode = match info.laser_mode {
                    Some(true) => "on",
                    Some(false) => "OFF ($32=0: jobs are refused)",
                    None => "unknown",
                };
                return Some(format!(
                    "laser: GRBL {}, laser mode {laser_mode}, $30={}",
                    info.version.as_deref().unwrap_or("?"),
                    info.s_max.map_or("?".to_owned(), |s| s.to_string())
                ));
            }
            Event::Disconnected { reason } => {
                self.view.port = None;
                self.view.ready = false;
                self.view.busy = None;
                self.view.state.clear();
                return Some(format!("laser: {reason}"));
            }
            Event::PermissionDenied { port, help } => {
                self.view.port = None;
                self.view.notice = Some(help);
                return Some(format!("laser: permission denied for {port}"));
            }
            Event::Status { state, position } => {
                self.view.state = state.to_string();
                if let Some(p) = position {
                    self.view.position = Some([p[0], p[1]]);
                }
            }
            Event::Alarm { code, message } => {
                let text = match code {
                    Some(code) => format!("Alarm {code}: {message}"),
                    None => format!("Alarm: {message}"),
                };
                self.view.alarm = Some(text.clone());
                return Some(format!("laser: {text}"));
            }
            Event::Stalled => {
                return Some("laser: the controller stopped answering".to_owned());
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
                self.view.busy = None;
                self.started = None;
                if let Outcome::Failed(reason) = &outcome {
                    self.view.notice = Some(format!("The {kind} failed: {reason}"));
                }
                return Some(format!("laser: {kind} {outcome}"));
            }
            Event::Rejected(reason) => {
                if self.view.busy.as_ref().is_some_and(|b| b.total == 0) {
                    // The stream we showed as started was refused.
                    self.view.busy = None;
                    self.started = None;
                }
                return Some(format!("laser: refused: {reason}"));
            }
        }
        None
    }

    /// Whether the UI should keep repainting to show live status.
    pub fn connected(&self) -> bool {
        self.view.port.is_some()
    }
}

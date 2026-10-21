//! The streaming state machine (`docs/spec/device.md`). No I/O and no clock:
//! the caller passes requests, received lines and the time, and collects the
//! bytes to write and the events.

use std::collections::VecDeque;

use crate::grbl::{
    CYCLE_START, FEED_HOLD, MAX_LINE, RX_BUFFER_SIZE, SOFT_RESET, SPINDLE_STOP, STATUS_QUERY,
    alarm_text, error_text,
};
use crate::reply::{self, Reply, StatusReport};
use crate::{Event, Info, MachineState, Outcome, Request, StreamKind};

/// How lines are paced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Protocol {
    /// Fill GRBL's receive buffer, counting bytes not yet answered.
    #[default]
    CharacterCounting,
    /// One line at a time ("ping-pong").
    SendResponse,
}

/// Timings and pacing (`docs/spec/device.md`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// Streaming protocol.
    pub protocol: Protocol,
    /// Status query interval, s.
    pub poll_interval: f64,
    /// No status reply for this long is a stall, s.
    pub stall_timeout: f64,
    /// Wait this long for the banner, and again for a status reply, s.
    pub handshake_timeout: f64,
    /// Send soft reset this long after a stop's feed hold at the latest, s.
    pub reset_fallback: f64,
    /// Longest homing cycle, s.
    pub homing_timeout: f64,
    /// Minimum time between progress events, s.
    pub progress_interval: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            protocol: Protocol::CharacterCounting,
            poll_interval: 0.2,
            stall_timeout: 2.0,
            handshake_timeout: 3.0,
            reset_fallback: 1.0,
            homing_timeout: 90.0,
            progress_interval: 0.1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Origin {
    Query,
    Stream(usize),
    Console,
    Jog,
    Home,
    Unlock,
}

#[derive(Debug, Clone)]
struct Line {
    text: String,
    origin: Origin,
    /// Must be sent alone (send-response).
    alone: bool,
}

#[derive(Debug, Clone, Copy)]
struct Pending {
    len: usize,
    origin: Origin,
    alone: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Phase {
    /// Waiting for the banner; `probed` once `?` was sent instead.
    Banner {
        since: f64,
        probed: bool,
    },
    /// `$I` and `$$` are outstanding.
    Handshake {
        since: f64,
    },
    Ready,
    Closed,
}

#[derive(Debug, Clone)]
struct Stream {
    kind: StreamKind,
    lines: Vec<String>,
    next: usize,
    acked: usize,
    uses_m3: bool,
    paused: bool,
    spindle_stopped: bool,
    /// When the last progress event went out.
    last_progress: f64,
}

/// The connection's state machine.
#[derive(Debug, Clone)]
pub struct Controller {
    settings: Settings,
    port: String,
    phase: Phase,
    now: f64,
    out: Vec<u8>,
    events: Vec<Event>,
    info: Info,
    queue: VecDeque<Line>,
    pending: VecDeque<Pending>,
    stream: Option<Stream>,
    /// Feed hold sent for a stop; soft reset follows.
    stopping: Option<f64>,
    /// Soft reset sent; replies until the banner (or this deadline) are
    /// leftovers.
    resetting: Option<f64>,
    state: Option<MachineState>,
    position: Option<[f64; 3]>,
    wco: [f64; 3],
    last_poll: f64,
    last_status: f64,
    stalled: bool,
    homing: Option<f64>,
    ping_pong: bool,
    /// The next status report goes to the console (the user typed `?`).
    show_next_status: bool,
}

/// Splits G-code text into the lines to send: comments and blank lines
/// dropped, whitespace trimmed.
fn program_lines(gcode: &str) -> Vec<String> {
    gcode
        .lines()
        .map(|line| {
            let mut out = String::new();
            let mut depth = 0;
            for c in line.chars() {
                match c {
                    '(' => depth += 1,
                    ')' if depth > 0 => depth -= 1,
                    ';' if depth == 0 => break,
                    c if depth == 0 => out.push(c),
                    _ => {}
                }
            }
            out.trim().to_owned()
        })
        .filter(|line| !line.is_empty())
        .collect()
}

/// Whether a line contains the word `M3` (constant laser power).
fn has_m3(line: &str) -> bool {
    let upper = line.to_ascii_uppercase();
    let bytes = upper.as_bytes();
    bytes
        .windows(2)
        .enumerate()
        .any(|(i, w)| w == b"M3" && !bytes.get(i + 2).is_some_and(u8::is_ascii_digit))
        || upper.contains("M03")
}

/// `$` commands other than jogging, and EEPROM writes, go one at a time.
fn needs_alone(line: &str) -> bool {
    let upper = line.to_ascii_uppercase();
    (upper.starts_with('$') && !upper.starts_with("$J="))
        || upper.starts_with("G10")
        || upper.starts_with("G28.1")
        || upper.starts_with("G30.1")
}

fn num(v: f64) -> String {
    let text = format!("{v:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
        "0".to_owned()
    } else {
        text.to_owned()
    }
}

/// The `$J=` line for a jog.
#[must_use]
pub fn jog_line(x: f64, y: f64, feed: f64, relative: bool) -> String {
    format!(
        "$J=G21 G9{} X{} Y{} F{}",
        if relative { 1 } else { 0 },
        num(x),
        num(y),
        num(feed)
    )
}

impl Controller {
    /// A controller for a port that has just been opened at time `now`.
    #[must_use]
    pub fn new(port: impl Into<String>, settings: Settings, now: f64) -> Self {
        Self {
            settings,
            port: port.into(),
            phase: Phase::Banner {
                since: now,
                probed: false,
            },
            now,
            out: Vec::new(),
            events: Vec::new(),
            info: Info {
                rx_buffer: RX_BUFFER_SIZE,
                ..Info::default()
            },
            queue: VecDeque::new(),
            pending: VecDeque::new(),
            stream: None,
            stopping: None,
            resetting: None,
            state: None,
            position: None,
            wco: [0.0; 3],
            last_poll: f64::NEG_INFINITY,
            last_status: now,
            stalled: false,
            homing: None,
            ping_pong: false,
            show_next_status: false,
        }
    }

    /// Takes the bytes to write to the port.
    pub fn take_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.out)
    }

    /// Takes the events produced so far.
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// What the handshake learned.
    #[must_use]
    pub fn info(&self) -> &Info {
        &self.info
    }

    /// The last reported machine state.
    #[must_use]
    pub fn state(&self) -> Option<MachineState> {
        self.state
    }

    /// The last reported work position.
    #[must_use]
    pub fn position(&self) -> Option<[f64; 3]> {
        self.position
    }

    /// Whether a stream is running.
    #[must_use]
    pub fn is_streaming(&self) -> bool {
        self.stream.is_some()
    }

    /// Whether the handshake has finished and the port is usable.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.phase == Phase::Ready
    }

    /// Whether the controller gave up on the port (the caller closes it).
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.phase == Phase::Closed
    }

    /// Lines sent and not yet answered.
    #[must_use]
    pub fn outstanding(&self) -> usize {
        self.pending.len()
    }

    /// Whether lines are paced one at a time now.
    #[must_use]
    pub fn is_ping_pong(&self) -> bool {
        self.ping_pong || self.settings.protocol == Protocol::SendResponse
    }

    fn emit(&mut self, event: Event) {
        self.events.push(event);
    }

    fn reject(&mut self, reason: impl Into<String>) {
        self.emit(Event::Rejected(reason.into()));
    }

    fn console(&mut self, sent: bool, text: &str, routine: bool) {
        self.emit(Event::Console {
            sent,
            text: text.to_owned(),
            routine,
        });
    }

    /// Handles a request at time `now`. `Connect` and `Disconnect` belong to
    /// the device thread; here `Disconnect` stops everything.
    pub fn request(&mut self, request: Request, now: f64) {
        self.now = now;
        match request {
            Request::Connect { .. } => self.reject("already connected"),
            Request::Disconnect => self.shutdown(),
            Request::Pause => self.pause(),
            Request::Resume => self.resume(),
            Request::Stop => self.stop(Outcome::Stopped),
            Request::Stream { kind, gcode } => self.start(kind, &gcode),
            Request::Jog {
                x,
                y,
                feed,
                relative,
            } => {
                if self.can_move("jog", true) {
                    self.queue_line(jog_line(x, y, feed, relative), Origin::Jog);
                }
            }
            Request::Home => {
                if self.can_move("home", false) {
                    self.queue_line("$H".to_owned(), Origin::Home);
                }
            }
            Request::Unlock => {
                if self.check_ready("unlock") {
                    self.queue_line("$X".to_owned(), Origin::Unlock);
                }
            }
            Request::Console(line) => self.console_line(&line),
        }
        self.pump();
    }

    fn check_ready(&mut self, what: &str) -> bool {
        if self.phase != Phase::Ready {
            self.reject(format!("cannot {what}: the controller is not ready"));
            false
        } else if self.stream.is_some() || self.stopping.is_some() || self.resetting.is_some() {
            self.reject(format!("cannot {what} while a job or frame is running"));
            false
        } else {
            true
        }
    }

    /// Moving is allowed when idle (and, for jogs, while jogging; homing
    /// also from an alarm).
    fn can_move(&mut self, what: &str, jog: bool) -> bool {
        if !self.check_ready(what) {
            return false;
        }
        let ok = match self.state {
            Some(MachineState::Idle) => true,
            Some(MachineState::Jog) => jog,
            Some(MachineState::Alarm) => what == "home",
            _ => false,
        };
        if !ok || self.homing.is_some() {
            let state = self.state.map_or("unknown".to_owned(), |s| s.to_string());
            self.reject(format!("cannot {what}: the machine is not idle ({state})"));
        }
        ok && self.homing.is_none()
    }

    fn queue_line(&mut self, text: String, origin: Origin) {
        let alone = needs_alone(&text);
        self.queue.push_back(Line {
            text,
            origin,
            alone,
        });
    }

    fn console_line(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() {
            return;
        }
        let realtime = match line {
            "?" => Some(STATUS_QUERY),
            "!" => Some(FEED_HOLD),
            "~" => Some(CYCLE_START),
            _ => None,
        };
        if let Some(byte) = realtime {
            if self.phase == Phase::Ready {
                self.console(true, line, false);
                self.out.push(byte);
                self.show_next_status = byte == STATUS_QUERY;
            }
            return;
        }
        if !self.check_ready("send a console line") {
            return;
        }
        if line.len() > MAX_LINE {
            self.reject(format!("the line is longer than {MAX_LINE} characters"));
            return;
        }
        let origin = if line.eq_ignore_ascii_case("$H") {
            Origin::Home
        } else {
            Origin::Console
        };
        self.queue_line(line.to_owned(), origin);
    }

    fn start(&mut self, kind: StreamKind, gcode: &str) {
        if !self.check_ready(&format!("start the {kind}")) {
            return;
        }
        if self.state != Some(MachineState::Idle) || self.homing.is_some() {
            let state = self.state.map_or("unknown".to_owned(), |s| s.to_string());
            self.reject(format!(
                "cannot start the {kind}: the machine is not idle ({state})"
            ));
            return;
        }
        if !self.queue.is_empty() || !self.pending.is_empty() {
            self.reject(format!(
                "cannot start the {kind}: earlier commands are still running"
            ));
            return;
        }
        if kind == StreamKind::Job && self.info.laser_mode != Some(true) {
            self.reject(match self.info.laser_mode {
                Some(false) => "laser mode is off ($32=0): the laser would stay on during travel moves; set $32=1 in the console",
                _ => "laser mode ($32) could not be read, so the job was not started",
            });
            return;
        }
        let lines = program_lines(gcode);
        if lines.is_empty() {
            self.reject(format!("the {kind} has no lines to send"));
            return;
        }
        if let Some((i, _)) = lines.iter().enumerate().find(|(_, l)| l.len() > MAX_LINE) {
            self.reject(format!(
                "line {} is longer than {MAX_LINE} characters",
                i + 1
            ));
            return;
        }
        let uses_m3 = lines.iter().any(|l| has_m3(l));
        let total = lines.len();
        self.stream = Some(Stream {
            kind,
            lines,
            next: 0,
            acked: 0,
            uses_m3,
            paused: false,
            spindle_stopped: false,
            last_progress: f64::NEG_INFINITY,
        });
        self.emit(Event::Progress {
            kind,
            acknowledged: 0,
            total,
        });
    }

    fn pause(&mut self) {
        if let Some(stream) = self.stream.as_mut()
            && !stream.paused
        {
            stream.paused = true;
            self.out.push(FEED_HOLD);
        }
    }

    fn resume(&mut self) {
        let paused = self.stream.as_ref().is_some_and(|s| s.paused);
        if paused || self.state == Some(MachineState::Hold) {
            if let Some(stream) = self.stream.as_mut() {
                stream.paused = false;
                stream.spindle_stopped = false;
            }
            self.out.push(CYCLE_START);
        }
    }

    /// Ends the stream with `outcome`, holds and schedules the reset.
    fn stop(&mut self, outcome: Outcome) {
        if self.phase != Phase::Ready {
            return;
        }
        self.queue.clear();
        self.homing = None;
        let moving = self.stream.is_some() || self.state.is_some_and(MachineState::is_moving);
        if let Some(stream) = self.stream.take() {
            self.emit(Event::Finished {
                kind: stream.kind,
                outcome,
            });
        }
        if moving {
            self.out.push(FEED_HOLD);
            self.stopping = Some(self.now);
        } else {
            self.send_reset();
        }
    }

    fn send_reset(&mut self) {
        self.out.push(SOFT_RESET);
        self.stopping = None;
        self.queue.clear();
        self.pending.clear();
        self.homing = None;
        self.resetting = Some(self.now + self.settings.handshake_timeout);
    }

    fn shutdown(&mut self) {
        if self.stream.is_some() || self.state.is_some_and(MachineState::is_moving) {
            if let Some(stream) = self.stream.take() {
                self.emit(Event::Finished {
                    kind: stream.kind,
                    outcome: Outcome::Stopped,
                });
            }
            self.out.push(FEED_HOLD);
            self.out.push(SOFT_RESET);
        }
        self.phase = Phase::Closed;
    }

    /// The port failed: whatever was running has stopped.
    pub fn port_lost(&mut self, reason: &str) {
        if let Some(stream) = self.stream.take() {
            self.emit(Event::Finished {
                kind: stream.kind,
                outcome: Outcome::Failed(format!("disconnected: {reason}")),
            });
        }
        self.phase = Phase::Closed;
        self.emit(Event::Disconnected {
            reason: reason.to_owned(),
        });
    }

    /// Ends a running stream after a fault (error or stall): feed hold, then
    /// soft reset.
    fn fail(&mut self, reason: String) {
        if self.stream.is_some() {
            self.stop(Outcome::Failed(reason));
        }
    }

    /// Writes as many queued lines as the protocol allows.
    fn pump(&mut self) {
        if self.phase == Phase::Closed || matches!(self.phase, Phase::Banner { .. }) {
            return;
        }
        loop {
            if self.queue.is_empty() && self.stopping.is_none() && self.resetting.is_none() {
                self.refill_from_stream();
            }
            let Some(line) = self.queue.front() else {
                return;
            };
            let len = line.text.len() + 1;
            let used: usize = self.pending.iter().map(|p| p.len).sum();
            let alone = line.alone || self.is_ping_pong();
            let blocked = self.pending.iter().any(|p| p.alone)
                || (alone && !self.pending.is_empty())
                || used + len > self.info.rx_buffer;
            if blocked {
                return;
            }
            let Some(line) = self.queue.pop_front() else {
                return;
            };
            self.out.extend_from_slice(line.text.as_bytes());
            self.out.push(b'\n');
            self.pending.push_back(Pending {
                len,
                origin: line.origin,
                alone: line.alone,
            });
            let routine = matches!(line.origin, Origin::Stream(_) | Origin::Query);
            self.console(true, &line.text, routine);
            if line.origin == Origin::Home {
                self.homing = Some(self.now);
            }
        }
    }

    fn refill_from_stream(&mut self) {
        if let Some(stream) = self.stream.as_mut()
            && let Some(text) = stream.lines.get(stream.next)
        {
            let line = Line {
                text: text.clone(),
                origin: Origin::Stream(stream.next),
                alone: false,
            };
            stream.next += 1;
            self.queue.push_back(line);
        }
    }

    /// Handles one received line at time `now`.
    pub fn received(&mut self, line: &str, now: f64) {
        self.now = now;
        let line = line.trim();
        if line.is_empty() || self.phase == Phase::Closed {
            return;
        }
        let parsed = reply::parse(line);
        let routine = match &parsed {
            Reply::Status(_) => !std::mem::take(&mut self.show_next_status),
            Reply::Ok => matches!(
                self.pending.front(),
                Some(Pending {
                    origin: Origin::Stream(_) | Origin::Query,
                    ..
                })
            ),
            Reply::Setting(..) | Reply::Options { .. } => {
                matches!(self.phase, Phase::Handshake { .. })
            }
            _ => false,
        };
        self.console(false, line, routine);
        match parsed {
            Reply::Ok => self.answer(None),
            Reply::Error(code) => self.answer(Some(code)),
            Reply::Alarm(code) => self.alarm(code),
            Reply::Status(report) => self.status(&report),
            Reply::Banner(version) => self.banner(version),
            Reply::Options { rx_buffer, .. } => {
                if let Some(size) = rx_buffer.filter(|&n| n >= 16) {
                    self.info.rx_buffer = size;
                }
            }
            Reply::Setting(number, value) => self.setting(number, &value),
            Reply::Message(_) | Reply::Other(_) => {}
        }
        self.pump();
    }

    fn setting(&mut self, number: u16, value: &str) {
        let Ok(v) = value.parse::<f64>() else {
            return;
        };
        match number {
            22 => self.info.homing = Some(v != 0.0),
            30 => self.info.s_max = Some(v),
            32 => self.info.laser_mode = Some(v != 0.0),
            130 => self.info.max_travel[0] = Some(v),
            131 => self.info.max_travel[1] = Some(v),
            _ => {}
        }
    }

    fn banner(&mut self, version: String) {
        self.info.version = Some(version);
        self.queue.clear();
        self.pending.clear();
        self.resetting = None;
        self.stopping = None;
        self.homing = None;
        if let Some(stream) = self.stream.take() {
            self.emit(Event::Finished {
                kind: stream.kind,
                outcome: Outcome::Failed("the controller reset".to_owned()),
            });
        }
        if let Phase::Banner { .. } = self.phase {
            self.begin_handshake();
        } else if let Phase::Handshake { .. } = self.phase {
            self.begin_handshake();
        }
    }

    fn begin_handshake(&mut self) {
        if !matches!(self.phase, Phase::Handshake { .. }) {
            self.emit(Event::Connected {
                port: self.port.clone(),
            });
        }
        self.phase = Phase::Handshake { since: self.now };
        self.queue_line("$I".to_owned(), Origin::Query);
        self.queue_line("$$".to_owned(), Origin::Query);
    }

    fn finish_handshake(&mut self) {
        self.phase = Phase::Ready;
        self.last_status = self.now;
        self.last_poll = f64::NEG_INFINITY;
        self.emit(Event::Ready(self.info.clone()));
    }

    fn answer(&mut self, error: Option<u16>) {
        let Some(done) = self.pending.pop_front() else {
            if self.resetting.is_none() && self.stream.is_some() && !self.ping_pong {
                // More answers than lines: the counts cannot be trusted.
                self.ping_pong = true;
                self.console(
                    false,
                    "(LaserCraft: unexpected reply; sending one line at a time from now on)",
                    false,
                );
            }
            return;
        };
        if self.resetting.is_some() {
            return;
        }
        match done.origin {
            Origin::Stream(i) => self.stream_answer(i, error),
            Origin::Query => {
                let outstanding = self.pending.iter().any(|p| p.origin == Origin::Query)
                    || self.queue.iter().any(|l| l.origin == Origin::Query);
                if matches!(self.phase, Phase::Handshake { .. }) && !outstanding {
                    self.finish_handshake();
                }
            }
            Origin::Home => {
                self.homing = None;
                self.last_status = self.now;
                match error {
                    // GRBL is idle once homing finishes.
                    None => self.state = Some(MachineState::Idle),
                    Some(code) => self.reject(format!(
                        "homing refused: error {code}: {}",
                        error_text(code)
                    )),
                }
            }
            Origin::Jog => {
                if let Some(code) = error {
                    self.reject(format!("jog refused: error {code}: {}", error_text(code)));
                }
            }
            Origin::Unlock if error.is_none() && self.state == Some(MachineState::Alarm) => {
                self.state = Some(MachineState::Idle);
            }
            Origin::Console | Origin::Unlock => {}
        }
    }

    fn stream_answer(&mut self, index: usize, error: Option<u16>) {
        let now = self.now;
        let interval = self.settings.progress_interval;
        let Some(stream) = self.stream.as_mut() else {
            return;
        };
        if let Some(code) = error {
            let text = stream.lines.get(index).cloned().unwrap_or_default();
            self.fail(format!(
                "line {} `{text}`: error {code}: {}",
                index + 1,
                error_text(code)
            ));
            return;
        }
        stream.acked += 1;
        let total = stream.lines.len();
        if stream.acked == total || now - stream.last_progress >= interval {
            stream.last_progress = now;
            let event = Event::Progress {
                kind: stream.kind,
                acknowledged: stream.acked,
                total,
            };
            self.emit(event);
        }
    }

    fn alarm(&mut self, code: u16) {
        self.state = Some(MachineState::Alarm);
        self.emit(Event::Alarm {
            code: Some(code),
            message: alarm_text(code).to_owned(),
        });
        self.homing = None;
        self.queue.clear();
        self.pending.clear();
        if let Some(stream) = self.stream.take() {
            self.emit(Event::Finished {
                kind: stream.kind,
                outcome: Outcome::Failed(format!("alarm {code}: {}", alarm_text(code))),
            });
            // Flush whatever GRBL still holds; the alarm stays.
            self.send_reset();
        }
        self.stopping = None;
    }

    fn status(&mut self, report: &StatusReport) {
        self.last_status = self.now;
        if self.stalled {
            self.stalled = false;
        }
        if let Phase::Banner { .. } = self.phase {
            // GRBL was already running: no banner, but it answers.
            self.begin_handshake();
        }
        if let Some(wco) = report.wco {
            self.wco = wco;
        }
        let position = report.wpos.or_else(|| {
            report
                .mpos
                .map(|m| [m[0] - self.wco[0], m[1] - self.wco[1], m[2] - self.wco[2]])
        });
        self.state = Some(report.state);
        self.position = position.or(self.position);
        self.emit(Event::Status {
            state: report.state,
            position: self.position,
        });
        let held = report.state == MachineState::Hold && report.substate == Some(0);
        if self.stopping.is_some()
            && (held || matches!(report.state, MachineState::Idle | MachineState::Alarm))
        {
            self.send_reset();
        }
        if let Some(stream) = self.stream.as_mut() {
            if held && stream.paused && stream.uses_m3 && !stream.spindle_stopped {
                stream.spindle_stopped = true;
                self.out.push(SPINDLE_STOP);
            }
            let done = stream.next == stream.lines.len()
                && stream.acked == stream.lines.len()
                && report.state == MachineState::Idle;
            if done {
                let kind = stream.kind;
                self.stream = None;
                self.emit(Event::Finished {
                    kind,
                    outcome: Outcome::Completed,
                });
            }
        }
    }

    /// Advances timers to `now`: polling, stalls, handshake and reset
    /// timeouts.
    pub fn tick(&mut self, now: f64) {
        self.now = now;
        match self.phase {
            Phase::Closed => return,
            Phase::Banner { since, probed } => {
                if now - since >= self.settings.handshake_timeout {
                    if probed {
                        self.phase = Phase::Closed;
                        self.emit(Event::Disconnected {
                            reason: "no reply from a GRBL controller".to_owned(),
                        });
                        return;
                    }
                    self.out.push(STATUS_QUERY);
                    self.phase = Phase::Banner {
                        since: now,
                        probed: true,
                    };
                }
                return;
            }
            Phase::Handshake { since } => {
                if now - since >= self.settings.handshake_timeout {
                    self.queue.retain(|l| l.origin != Origin::Query);
                    self.pending.clear();
                    self.console(
                        false,
                        "(LaserCraft: the controller did not answer $I/$$; settings unknown)",
                        false,
                    );
                    self.finish_handshake();
                }
            }
            Phase::Ready => {}
        }
        if let Some(deadline) = self.resetting
            && now >= deadline
        {
            self.resetting = None;
        }
        if let Some(since) = self.stopping
            && now - since >= self.settings.reset_fallback
        {
            self.send_reset();
        }
        if self.phase == Phase::Ready {
            self.poll(now);
        }
        self.pump();
    }

    fn poll(&mut self, now: f64) {
        if let Some(since) = self.homing {
            if now - since < self.settings.homing_timeout {
                return; // GRBL answers no status while homing
            }
            self.homing = None;
            self.last_status = now;
            self.reject("homing did not finish in time");
        }
        if now - self.last_poll >= self.settings.poll_interval {
            self.out.push(STATUS_QUERY);
            self.last_poll = now;
        }
        if !self.stalled && now - self.last_status >= self.settings.stall_timeout {
            self.stalled = true;
            self.emit(Event::Stalled);
            self.fail("the controller stopped answering".to_owned());
        }
    }
}

#[cfg(test)]
mod tests;

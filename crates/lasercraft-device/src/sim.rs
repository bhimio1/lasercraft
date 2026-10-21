//! A simulated GRBL 1.1 laser controller on simulated time
//! (`docs/spec/device.md`, "Simulator"). It has no I/O: bytes go in with
//! [`Sim::write`], time passes with [`Sim::advance`], replies come out of
//! [`Sim::read`]. Every stretch with the laser on is recorded.

use std::collections::{BTreeMap, VecDeque};

use crate::MachineState;
use crate::grbl::{
    CYCLE_START, FEED_HOLD, MAX_LINE, RX_BUFFER_SIZE, SOFT_RESET, SPINDLE_STOP, STATUS_QUERY,
};

/// Firmware version the simulator reports.
pub const VERSION: &str = "1.1h";

/// Simulation step, seconds.
const STEP: f64 = 0.001;

/// Positions may be this far outside the travel, mm.
const SLACK: f64 = 1e-6;

/// How the simulated machine is set up.
#[derive(Debug, Clone, PartialEq)]
pub struct SimConfig {
    /// X travel, mm (`$130`).
    pub width: f64,
    /// Y travel, mm (`$131`).
    pub height: f64,
    /// Laser mode (`$32`).
    pub laser_mode: bool,
    /// S for full power (`$30`).
    pub s_max: f64,
    /// Homing enabled (`$22`).
    pub homing: bool,
    /// Soft limits (`$20`): motion outside the travel raises alarm 2, jogs
    /// outside it are refused with error 15.
    pub soft_limits: bool,
    /// Start in alarm, as GRBL does at power-up with homing enabled.
    pub start_locked: bool,
    /// Rapid (`G0`) speed, mm/min.
    pub rapid: f64,
    /// Time a feed hold takes to stop, s.
    pub hold_time: f64,
    /// Time a homing cycle takes, s.
    pub homing_time: f64,
    /// Receive buffer size, bytes.
    pub rx_buffer: usize,
    /// Planner blocks.
    pub blocks: usize,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            width: 400.0,
            height: 400.0,
            laser_mode: true,
            s_max: 1000.0,
            homing: true,
            soft_limits: true,
            start_locked: false,
            rapid: 6000.0,
            hold_time: 0.05,
            homing_time: 2.0,
            rx_buffer: RX_BUFFER_SIZE,
            blocks: 15,
        }
    }
}

/// A stretch of time with the laser on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lit {
    /// When it switched on, s.
    pub start: f64,
    /// When it switched off, s.
    pub end: f64,
    /// Distance moved while on, mm.
    pub length: f64,
    /// The `S` value.
    pub s: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Spindle {
    Off,
    Constant,
    Dynamic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    Rapid,
    Feed { spindle: Spindle, s: f64 },
    Jog,
    Dwell(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Block {
    target: [f64; 2],
    /// mm/min.
    feed: f64,
    kind: Kind,
}

/// What a line parses to before it runs.
struct Parsed {
    words: Vec<(char, f64)>,
}

/// The simulated controller.
#[derive(Debug, Clone)]
pub struct Sim {
    cfg: SimConfig,
    now: f64,
    rx: VecDeque<u8>,
    overflow: usize,
    out: String,
    state: MachineState,
    /// While decelerating into a hold (or a jog cancel): when it stops.
    stopping_until: Option<f64>,
    planner: VecDeque<Block>,
    current: Option<Block>,
    pos: [f64; 2],
    absolute: bool,
    rapid_mode: bool,
    spindle: Spindle,
    s: f64,
    feed: Option<f64>,
    spindle_stopped: bool,
    settings: BTreeMap<u16, String>,
    reports: u64,
    homing_until: Option<f64>,
    silent: bool,
    connected: bool,
    log: Vec<Lit>,
    lit: Option<Lit>,
}

fn fmt3(v: f64) -> String {
    format!("{v:.3}")
}

impl Sim {
    /// A machine that has just been powered on: its banner is waiting to be
    /// read.
    #[must_use]
    pub fn new(cfg: SimConfig) -> Self {
        let mut settings = BTreeMap::new();
        settings.insert(10, "1".to_owned());
        settings.insert(20, u8::from(cfg.soft_limits).to_string());
        settings.insert(22, u8::from(cfg.homing).to_string());
        settings.insert(30, cfg.s_max.to_string());
        settings.insert(31, "0".to_owned());
        settings.insert(32, u8::from(cfg.laser_mode).to_string());
        settings.insert(130, cfg.width.to_string());
        settings.insert(131, cfg.height.to_string());
        let locked = cfg.start_locked;
        let mut sim = Self {
            cfg,
            now: 0.0,
            rx: VecDeque::new(),
            overflow: 0,
            out: String::new(),
            state: MachineState::Idle,
            stopping_until: None,
            planner: VecDeque::new(),
            current: None,
            pos: [0.0, 0.0],
            absolute: true,
            rapid_mode: true,
            spindle: Spindle::Off,
            s: 0.0,
            feed: None,
            spindle_stopped: false,
            settings,
            reports: 0,
            homing_until: None,
            silent: false,
            connected: true,
            log: Vec::new(),
            lit: None,
        };
        if locked {
            sim.state = MachineState::Alarm;
        }
        sim.banner();
        sim
    }

    fn say(&mut self, line: &str) {
        self.out.push_str(line);
        self.out.push_str("\r\n");
    }

    fn banner(&mut self) {
        self.say(&format!("Grbl {VERSION} ['$' for help]"));
        if self.state == MachineState::Alarm {
            self.say("[MSG:'$H'|'$X' to unlock]");
        }
    }

    /// Simulated time, s.
    #[must_use]
    pub fn now(&self) -> f64 {
        self.now
    }

    /// Machine position, mm.
    #[must_use]
    pub fn position(&self) -> [f64; 2] {
        self.pos
    }

    /// Current state.
    #[must_use]
    pub fn state(&self) -> MachineState {
        self.state
    }

    /// Bytes dropped because the receive buffer was full (a streaming bug).
    #[must_use]
    pub fn overflows(&self) -> usize {
        self.overflow
    }

    /// Whether the laser is on right now.
    #[must_use]
    pub fn is_lit(&self) -> bool {
        self.lit.is_some()
    }

    /// Every stretch with the laser on so far, including one still going.
    #[must_use]
    pub fn laser_log(&self) -> Vec<Lit> {
        self.log.iter().copied().chain(self.lit).collect()
    }

    /// Total distance moved with the laser on, mm.
    #[must_use]
    pub fn lit_length(&self) -> f64 {
        self.laser_log().iter().map(|l| l.length).sum()
    }

    /// A setting's value, as `$$` prints it.
    #[must_use]
    pub fn setting(&self, number: u16) -> Option<&str> {
        self.settings.get(&number).map(String::as_str)
    }

    /// Whether the simulated cable is still plugged in.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Takes everything the controller has printed.
    pub fn read(&mut self) -> String {
        std::mem::take(&mut self.out)
    }

    /// Fault injection: stop answering anything (a stall).
    pub fn set_silent(&mut self, silent: bool) {
        self.silent = silent;
        if silent {
            self.out.clear();
        }
    }

    /// Fault injection: unplug the cable. Motion stops (as if powered off).
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.halt();
    }

    /// Raises an alarm now, as a limit switch would: motion stops at once and
    /// everything buffered is dropped.
    pub fn raise_alarm(&mut self, code: u16) {
        self.halt();
        self.state = MachineState::Alarm;
        self.say(&format!("ALARM:{code}"));
    }

    fn halt(&mut self) {
        self.end_lit();
        self.rx.clear();
        self.planner.clear();
        self.current = None;
        self.stopping_until = None;
        self.homing_until = None;
        self.spindle_stopped = false;
    }

    /// Receives bytes from the host.
    pub fn write(&mut self, bytes: &[u8]) {
        if !self.connected {
            return;
        }
        for &b in bytes {
            match b {
                STATUS_QUERY => self.status_report(),
                FEED_HOLD => self.feed_hold(),
                CYCLE_START => self.cycle_start(),
                SOFT_RESET => self.reset(),
                SPINDLE_STOP => {
                    if self.state == MachineState::Hold && self.stopping_until.is_none() {
                        self.spindle_stopped = !self.spindle_stopped;
                    }
                }
                b'\r' => {}
                _ if self.rx.len() < self.cfg.rx_buffer => self.rx.push_back(b),
                _ => self.overflow += 1,
            }
        }
        self.process_input();
    }

    fn status_report(&mut self) {
        if self.silent || self.state == MachineState::Home {
            return;
        }
        let state = match self.state {
            MachineState::Hold if self.stopping_until.is_some() => "Hold:1".to_owned(),
            MachineState::Hold => "Hold:0".to_owned(),
            other => other.to_string(),
        };
        let feed = match (self.moving(), self.current) {
            (true, Some(b)) => b.feed,
            _ => 0.0,
        };
        let s = self.lit.map_or(0.0, |l| l.s);
        let mut line = format!(
            "<{state}|MPos:{},{},0.000|FS:{feed},{s}",
            fmt3(self.pos[0]),
            fmt3(self.pos[1])
        );
        if self.reports.is_multiple_of(10) {
            line.push_str("|WCO:0.000,0.000,0.000");
        }
        line.push('>');
        self.reports += 1;
        self.say(&line);
    }

    fn feed_hold(&mut self) {
        match self.state {
            MachineState::Run => {
                self.state = MachineState::Hold;
                self.stopping_until = Some(self.now + self.cfg.hold_time);
            }
            MachineState::Jog => {
                // A hold cancels a jog: decelerate, drop queued jogs, idle.
                self.planner.clear();
                self.stopping_until = Some(self.now + self.cfg.hold_time);
            }
            MachineState::Idle => self.state = MachineState::Hold,
            _ => {}
        }
    }

    fn cycle_start(&mut self) {
        if self.state == MachineState::Hold && self.stopping_until.is_none() {
            self.spindle_stopped = false;
            self.state = if self.current.is_some() || !self.planner.is_empty() {
                MachineState::Run
            } else {
                MachineState::Idle
            };
        }
    }

    fn moving(&self) -> bool {
        match self.state {
            MachineState::Run | MachineState::Jog => true,
            MachineState::Hold => self.stopping_until.is_some(),
            _ => false,
        }
    }

    fn reset(&mut self) {
        let alarm = if self.state == MachineState::Home {
            Some(6)
        } else if self.moving() && self.current.is_some() {
            Some(3)
        } else {
            None
        };
        self.halt();
        self.absolute = true;
        self.rapid_mode = true;
        self.spindle = Spindle::Off;
        self.s = 0.0;
        self.feed = None;
        if let Some(code) = alarm {
            self.say(&format!("ALARM:{code}"));
            self.state = MachineState::Alarm;
        } else if self.state != MachineState::Alarm {
            self.state = MachineState::Idle;
        }
        self.say("");
        self.banner();
    }

    /// Runs complete lines from the receive buffer while they can run.
    fn process_input(&mut self) {
        while self.connected && self.state != MachineState::Home {
            let Some(end) = self.rx.iter().position(|&b| b == b'\n') else {
                return;
            };
            let line: String = self.rx.iter().take(end).map(|&b| char::from(b)).collect();
            let Some(replies) = self.execute(&line) else {
                return; // planner full: the line waits in the buffer
            };
            // An alarm raised by the line has already emptied the buffer.
            if self.rx.len() > end {
                self.rx.drain(..=end);
            }
            if !self.silent {
                for reply in replies {
                    self.say(&reply);
                }
            }
        }
    }

    /// Runs one line. `None` means it must wait for a free planner block.
    fn execute(&mut self, raw: &str) -> Option<Vec<String>> {
        let mut clean = String::new();
        let mut in_comment = false;
        for c in raw.chars() {
            match c {
                '(' => in_comment = true,
                ')' => in_comment = false,
                ';' if !in_comment => break,
                c if in_comment || c.is_whitespace() => {}
                c => clean.push(c.to_ascii_uppercase()),
            }
        }
        if raw.len() > MAX_LINE {
            return Some(vec!["error:11".to_owned()]);
        }
        if clean.is_empty() {
            return Some(vec!["ok".to_owned()]);
        }
        if let Some(rest) = clean.strip_prefix('$') {
            return self.system(rest);
        }
        self.gcode(&clean)
    }

    fn system(&mut self, rest: &str) -> Option<Vec<String>> {
        let ok = || vec!["ok".to_owned()];
        let err = |n: u16| Some(vec![format!("error:{n}")]);
        let idle = matches!(self.state, MachineState::Idle | MachineState::Alarm);
        if let Some(jog) = rest.strip_prefix("J=") {
            return self.jog(jog);
        }
        match rest {
            "$" => {
                if !idle {
                    return err(8);
                }
                let mut lines: Vec<String> = self
                    .settings
                    .iter()
                    .map(|(n, v)| format!("${n}={v}"))
                    .collect();
                lines.push("ok".to_owned());
                Some(lines)
            }
            "I" => Some(vec![
                format!("[VER:{VERSION}.20190825:]"),
                format!("[OPT:V,{},{}]", self.cfg.blocks, self.cfg.rx_buffer),
                "ok".to_owned(),
            ]),
            "G" => {
                let spindle = match self.spindle {
                    Spindle::Off => 5,
                    Spindle::Constant => 3,
                    Spindle::Dynamic => 4,
                };
                Some(vec![
                    format!(
                        "[GC:G{} G54 G17 G21 G{} G94 M{spindle} M9 T0 F{} S{}]",
                        u8::from(!self.rapid_mode),
                        if self.absolute { 90 } else { 91 },
                        self.feed.unwrap_or(0.0),
                        self.s
                    ),
                    "ok".to_owned(),
                ])
            }
            "X" => {
                let mut lines = Vec::new();
                if self.state == MachineState::Alarm {
                    self.state = MachineState::Idle;
                    lines.push("[MSG:Caution: Unlocked]".to_owned());
                }
                lines.push("ok".to_owned());
                Some(lines)
            }
            "H" => {
                if !self.cfg.homing {
                    return err(5);
                }
                if !idle {
                    return err(8);
                }
                self.state = MachineState::Home;
                self.homing_until = Some(self.now + self.cfg.homing_time);
                Some(Vec::new()) // `ok` comes when homing finishes
            }
            _ => {
                let Some((key, value)) = rest.split_once('=') else {
                    return err(3);
                };
                let (Ok(number), Ok(v)) = (key.parse::<u16>(), value.parse::<f64>()) else {
                    return err(3);
                };
                if !idle {
                    return err(8);
                }
                match number {
                    32 => self.cfg.laser_mode = v != 0.0,
                    30 => self.cfg.s_max = v,
                    _ => {}
                }
                self.settings.insert(number, value.to_owned());
                Some(ok())
            }
        }
    }

    fn parse(text: &str) -> Result<Parsed, u16> {
        let mut words = Vec::new();
        let mut chars = text.chars().peekable();
        while let Some(letter) = chars.next() {
            if !letter.is_ascii_alphabetic() {
                return Err(1);
            }
            let mut number = String::new();
            while let Some(d) = chars.next_if(|d| d.is_ascii_digit() || *d == '.' || *d == '-') {
                number.push(d);
            }
            let value: f64 = number.parse().map_err(|_| 2u16)?;
            words.push((letter, value));
        }
        Ok(Parsed { words })
    }

    fn outside(&self, p: [f64; 2]) -> bool {
        p[0] < -SLACK
            || p[1] < -SLACK
            || p[0] > self.cfg.width + SLACK
            || p[1] > self.cfg.height + SLACK
    }

    fn target(&self, words: &[(char, f64)], absolute: bool) -> Option<[f64; 2]> {
        let mut target = if absolute { self.end_pos() } else { [0.0, 0.0] };
        let mut any = false;
        for &(letter, value) in words {
            match letter {
                'X' => target[0] = value,
                'Y' => target[1] = value,
                _ => continue,
            }
            any = true;
        }
        if !absolute {
            target = [self.end_pos()[0] + target[0], self.end_pos()[1] + target[1]];
        }
        any.then_some(target)
    }

    /// Where the head will be once everything queued has run.
    fn end_pos(&self) -> [f64; 2] {
        self.planner
            .back()
            .or(self.current.as_ref())
            .map_or(self.pos, |b| b.target)
    }

    fn jog(&mut self, text: &str) -> Option<Vec<String>> {
        let err = |n: u16| Some(vec![format!("error:{n}")]);
        match self.state {
            MachineState::Idle | MachineState::Jog => {}
            MachineState::Alarm => return err(9),
            _ => return err(8),
        }
        let parsed = match Self::parse(text) {
            Ok(p) => p,
            Err(code) => return err(code),
        };
        let mut absolute = self.absolute;
        let mut feed = None;
        for &(letter, value) in &parsed.words {
            match (letter, value) {
                ('G', 90.0) => absolute = true,
                ('G', 91.0) => absolute = false,
                ('G', 20.0 | 21.0 | 53.0) | ('X' | 'Y' | 'Z' | 'N', _) => {}
                ('F', f) if f > 0.0 => feed = Some(f),
                _ => return err(16),
            }
        }
        let Some(feed) = feed else {
            return err(16);
        };
        let Some(target) = self.target(&parsed.words, absolute) else {
            return err(16);
        };
        if self.cfg.soft_limits && self.outside(target) {
            return err(15);
        }
        if self.planner.len() >= self.cfg.blocks {
            return None;
        }
        self.planner.push_back(Block {
            target,
            feed,
            kind: Kind::Jog,
        });
        self.state = MachineState::Jog;
        Some(vec!["ok".to_owned()])
    }

    fn gcode(&mut self, text: &str) -> Option<Vec<String>> {
        let err = |n: u16| Some(vec![format!("error:{n}")]);
        if matches!(self.state, MachineState::Alarm | MachineState::Jog) {
            return err(9);
        }
        let parsed = match Self::parse(text) {
            Ok(p) => p,
            Err(code) => return err(code),
        };
        let mut rapid = self.rapid_mode;
        let mut absolute = self.absolute;
        let mut spindle = self.spindle;
        let mut s = self.s;
        let mut feed = self.feed;
        let mut dwell = None;
        let mut moves = false;
        for &(letter, value) in &parsed.words {
            let whole = (value.fract() == 0.0).then_some(value as i64);
            match (letter, whole) {
                ('G', Some(0)) => rapid = true,
                ('G', Some(1)) => rapid = false,
                ('G', Some(4)) => dwell = Some(0.0),
                ('G', Some(17 | 21 | 54 | 94)) | ('M', Some(7..=9)) | ('Z', _) => {}
                ('G', Some(90)) => absolute = true,
                ('G', Some(91)) => absolute = false,
                ('M', Some(3)) => spindle = Spindle::Constant,
                ('M', Some(4)) => spindle = Spindle::Dynamic,
                ('M', Some(5)) => spindle = Spindle::Off,
                ('S', _) if value >= 0.0 => s = value.min(self.cfg.s_max),
                ('F', _) if value > 0.0 => feed = Some(value),
                ('P', _) if value >= 0.0 => {
                    if dwell.is_some() {
                        dwell = Some(value);
                    }
                }
                ('X' | 'Y', _) => moves = true,
                ('S' | 'F' | 'P', _) => return err(4),
                _ => return err(20),
            }
        }
        if (moves || dwell.is_some()) && self.planner.len() >= self.cfg.blocks {
            return None;
        }
        let target = if moves {
            self.target(&parsed.words, absolute)
        } else {
            None
        };
        if target.is_some() && !rapid && feed.is_none() {
            return err(22);
        }
        self.rapid_mode = rapid;
        self.absolute = absolute;
        self.spindle = spindle;
        self.s = s;
        self.feed = feed;
        if let Some(seconds) = dwell {
            self.queue(Block {
                target: self.end_pos(),
                feed: 0.0,
                kind: Kind::Dwell(seconds),
            });
        }
        if let Some(target) = target {
            if self.cfg.soft_limits && self.outside(target) {
                self.raise_alarm(2);
                return Some(Vec::new());
            }
            let block = if rapid {
                Block {
                    target,
                    feed: self.cfg.rapid,
                    kind: Kind::Rapid,
                }
            } else {
                Block {
                    target,
                    feed: feed.unwrap_or(0.0),
                    kind: Kind::Feed { spindle, s },
                }
            };
            self.queue(block);
        }
        Some(vec!["ok".to_owned()])
    }

    fn queue(&mut self, block: Block) {
        self.planner.push_back(block);
        if self.state == MachineState::Idle {
            self.state = MachineState::Run;
        }
    }

    /// The `S` the laser is firing at during the next step, if it is on.
    fn lit_power(&self) -> Option<f64> {
        if !self.cfg.laser_mode {
            // Without laser mode the output follows the spindle, moving or not.
            return (self.spindle != Spindle::Off && self.s > 0.0).then_some(self.s);
        }
        let Kind::Feed { spindle, s } = self.current?.kind else {
            return None;
        };
        if spindle == Spindle::Off || s <= 0.0 {
            return None;
        }
        if self.moving() {
            return Some(s);
        }
        // Held mid-cut: M3 keeps the power on unless spindle stop is used.
        (self.state == MachineState::Hold && spindle == Spindle::Constant && !self.spindle_stopped)
            .then_some(s)
    }

    fn end_lit(&mut self) {
        if let Some(lit) = self.lit.take() {
            self.log.push(lit);
        }
    }

    /// Lets `dt` seconds pass.
    pub fn advance(&mut self, dt: f64) {
        let end = self.now + dt;
        while self.now < end - 1e-12 {
            let h = (end - self.now).min(STEP);
            self.step(h);
            self.now += h;
            self.process_input();
        }
    }

    fn step(&mut self, h: f64) {
        if !self.connected {
            return;
        }
        if let Some(until) = self.homing_until {
            if self.now + h >= until {
                self.homing_until = None;
                self.pos = [0.0, 0.0];
                self.state = MachineState::Idle;
                if !self.silent {
                    self.say("ok");
                }
            }
            return;
        }
        if let Some(until) = self.stopping_until
            && self.now >= until
        {
            self.stopping_until = None;
            if self.state == MachineState::Jog {
                self.current = None;
                self.planner.clear();
                self.state = MachineState::Idle;
            }
        }
        if self.moving() && self.current.is_none() {
            self.current = self.planner.pop_front();
            if self.current.is_none() && matches!(self.state, MachineState::Run | MachineState::Jog)
            {
                self.state = MachineState::Idle;
            }
        }
        let power = self.lit_power();
        let moved = if self.moving() { self.travel(h) } else { 0.0 };
        match (power, self.lit.as_mut()) {
            (Some(s), Some(lit)) if lit.s == s => {
                lit.end = self.now + h;
                lit.length += moved;
            }
            (Some(s), _) => {
                self.end_lit();
                self.lit = Some(Lit {
                    start: self.now,
                    end: self.now + h,
                    length: moved,
                    s,
                });
            }
            (None, _) => self.end_lit(),
        }
    }

    /// Moves along the current block for up to `h` seconds; returns the
    /// distance covered. The next block starts on the next step, so each lit
    /// stretch belongs to one block (at most one step is lost per block).
    fn travel(&mut self, h: f64) -> f64 {
        let Some(block) = self.current.as_mut() else {
            return 0.0;
        };
        if let Kind::Dwell(left) = &mut block.kind {
            *left -= h;
            if *left <= 0.0 {
                self.current = None;
            }
            return 0.0;
        }
        let dx = block.target[0] - self.pos[0];
        let dy = block.target[1] - self.pos[1];
        let remaining = dx.hypot(dy);
        let reach = block.feed / 60.0 * h;
        if reach >= remaining {
            self.pos = block.target;
            self.current = None;
            remaining
        } else {
            self.pos = [
                self.pos[0] + dx / remaining * reach,
                self.pos[1] + dy / remaining * reach,
            ];
            reach
        }
    }
}

#[cfg(test)]
mod tests;

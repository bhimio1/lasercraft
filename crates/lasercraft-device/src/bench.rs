//! A controller wired to the simulator on simulated time: what the device
//! thread does, minus the port and the clock.

use crate::controller::{Controller, Settings};
use crate::sim::{Sim, SimConfig};
use crate::{Event, Request, SIMULATOR_PORT};

/// Simulated time per step, s.
const STEP: f64 = 0.005;

/// A controller connected to a simulated GRBL.
#[derive(Debug, Clone)]
pub struct Bench {
    /// The simulated machine.
    pub sim: Sim,
    /// The streaming state machine.
    pub controller: Controller,
    /// Every event so far, oldest first.
    pub events: Vec<Event>,
    partial: String,
}

impl Bench {
    /// A freshly powered machine with a controller that has just opened the
    /// port.
    #[must_use]
    pub fn new(config: SimConfig, settings: Settings) -> Self {
        Self {
            sim: Sim::new(config),
            controller: Controller::new(SIMULATOR_PORT, settings, 0.0),
            events: Vec::new(),
            partial: String::new(),
        }
    }

    /// A bench that has finished the handshake.
    ///
    /// # Panics
    ///
    /// If the handshake does not finish within 10 simulated seconds.
    #[must_use]
    pub fn ready(config: SimConfig) -> Self {
        let mut bench = Self::new(config, Settings::default());
        assert!(
            bench.run_until(10.0, |e| matches!(e, Event::Ready(_))),
            "no handshake: {:?}",
            bench.events
        );
        // One status report so the state is known.
        assert!(bench.run_until(1.0, |e| matches!(e, Event::Status { .. })));
        bench
    }

    /// Simulated time, s.
    #[must_use]
    pub fn now(&self) -> f64 {
        self.sim.now()
    }

    /// Hands a request to the controller.
    pub fn request(&mut self, request: Request) {
        let now = self.now();
        self.controller.request(request, now);
        self.flush();
    }

    fn flush(&mut self) {
        let bytes = self.controller.take_output();
        if !bytes.is_empty() {
            self.sim.write(&bytes);
        }
        self.events.extend(self.controller.take_events());
    }

    /// One step: time passes, replies reach the controller, its output reaches
    /// the machine.
    pub fn step(&mut self) {
        self.sim.advance(STEP);
        let now = self.now();
        if !self.sim.is_connected() {
            if !self.controller.is_closed() {
                self.controller
                    .port_lost("the simulated cable was unplugged");
            }
            self.flush();
            return;
        }
        self.partial.push_str(&self.sim.read());
        while let Some(end) = self.partial.find('\n') {
            let line: String = self.partial.drain(..=end).collect();
            self.controller.received(&line, now);
        }
        self.controller.tick(now);
        self.flush();
    }

    /// Steps until an event matching `done` arrives (after the call) or
    /// `seconds` pass. Returns whether it arrived.
    pub fn run_until(&mut self, seconds: f64, mut done: impl FnMut(&Event) -> bool) -> bool {
        let from = self.events.len();
        let end = self.now() + seconds;
        while self.now() < end {
            self.step();
            if self.events[from..].iter().any(&mut done) {
                return true;
            }
        }
        false
    }

    /// Steps for `seconds`.
    pub fn run_for(&mut self, seconds: f64) {
        self.run_until(seconds, |_| false);
    }

    /// Events from index `from` on.
    #[must_use]
    pub fn events_since(&self, from: usize) -> &[Event] {
        &self.events[from.min(self.events.len())..]
    }
}

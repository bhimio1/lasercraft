use std::fmt;

/// The port name that opens the simulated machine instead of a serial port.
pub const SIMULATOR_PORT: &str = "simulator";

/// What a stream is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
    /// A job: the laser may fire.
    Job,
    /// Framing: laser-off moves only.
    Frame,
}

impl fmt::Display for StreamKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Job => "job",
            Self::Frame => "frame",
        })
    }
}

/// How a stream ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Every line ran and the machine is idle.
    Completed,
    /// The user stopped it.
    Stopped,
    /// An error, alarm, stall or disconnect ended it.
    Failed(String),
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Completed => f.write_str("completed"),
            Self::Stopped => f.write_str("stopped"),
            Self::Failed(reason) => write!(f, "failed: {reason}"),
        }
    }
}

/// What the handshake learned about the controller.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Info {
    /// Firmware version from the banner, such as `1.1h`.
    pub version: Option<String>,
    /// Receive buffer size in bytes.
    pub rx_buffer: usize,
    /// Laser mode (`$32`).
    pub laser_mode: Option<bool>,
    /// S value for full power (`$30`).
    pub s_max: Option<f64>,
    /// Maximum travel in X and Y (`$130`, `$131`), mm.
    pub max_travel: [Option<f64>; 2],
    /// Homing enabled (`$22`).
    pub homing: Option<bool>,
}

/// What the UI or CLI asks the device thread to do
/// (`docs/spec/device.md`, "Requests").
#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    /// Open a serial port, or [`SIMULATOR_PORT`].
    Connect {
        /// Port name, such as `/dev/ttyUSB0` or `COM3`.
        port: String,
        /// Baud rate.
        baud: u32,
    },
    /// Stop anything running and close the port.
    Disconnect,
    /// Stream a G-code program.
    Stream {
        /// Job or framing.
        kind: StreamKind,
        /// The program text.
        gcode: String,
    },
    /// Feed hold (`!`).
    Pause,
    /// Resume after a hold (`~`).
    Resume,
    /// Abort: stop streaming, feed hold, then soft reset (0x18).
    Stop,
    /// Jog with `$J=`, laser off.
    Jog {
        /// X distance (relative) or position (absolute), mm.
        x: f64,
        /// Y distance (relative) or position (absolute), mm.
        y: f64,
        /// Feed rate, mm/min.
        feed: f64,
        /// Relative (`G91`) or absolute (`G90`) coordinates.
        relative: bool,
    },
    /// Run the homing cycle (`$H`).
    Home,
    /// Clear an alarm lock (`$X`).
    Unlock,
    /// Send a line typed by the user.
    Console(String),
}

/// What the device thread reports back (`docs/spec/device.md`, "Events").
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The port is open and the controller answered.
    Connected {
        /// Port name.
        port: String,
    },
    /// The handshake finished; the machine can be used.
    Ready(Info),
    /// The connection ended. Streaming has stopped.
    Disconnected {
        /// Why, in words for the user.
        reason: String,
    },
    /// The OS refused to open the port. `help` says how to fix it.
    PermissionDenied {
        /// Port name.
        port: String,
        /// Platform-specific instructions (see [`crate::permissions`]).
        help: String,
    },
    /// A status report.
    Status {
        /// Machine state.
        state: MachineState,
        /// Work position in millimetres, if reported.
        position: Option<[f64; 3]>,
    },
    /// The controller is in alarm. Streaming has stopped.
    Alarm {
        /// GRBL alarm code, if known.
        code: Option<u16>,
        /// Description for the user.
        message: String,
    },
    /// No status reply arrived in time. Streaming has stopped.
    Stalled,
    /// A line sent or received, for the console.
    Console {
        /// Sent by us (`true`) or received.
        sent: bool,
        /// The line.
        text: String,
        /// Routine traffic (status reports, streamed lines and their `ok`),
        /// hidden unless the console shows everything.
        routine: bool,
    },
    /// Lines of the running stream that GRBL has accepted.
    Progress {
        /// Job or framing.
        kind: StreamKind,
        /// Lines answered `ok`.
        acknowledged: usize,
        /// Lines in the stream.
        total: usize,
    },
    /// A stream ended.
    Finished {
        /// Job or framing.
        kind: StreamKind,
        /// How it ended.
        outcome: Outcome,
    },
    /// A request was refused.
    Rejected(String),
}

/// The machine states GRBL reports at the start of a status report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineState {
    /// Ready.
    Idle,
    /// Executing motion.
    Run,
    /// Feed hold in progress or complete.
    Hold,
    /// Jogging.
    Jog,
    /// Alarm: motion is locked until the alarm is cleared.
    Alarm,
    /// Safety door open.
    Door,
    /// Check (dry-run) mode.
    Check,
    /// Homing cycle.
    Home,
    /// Sleeping.
    Sleep,
}

impl MachineState {
    /// Parses the state field of a status report, such as `Idle` or `Hold:0`
    /// (sub-states after `:` are ignored).
    #[must_use]
    pub fn parse(field: &str) -> Option<Self> {
        let name = field.split(':').next().unwrap_or(field);
        Some(match name {
            "Idle" => Self::Idle,
            "Run" => Self::Run,
            "Hold" => Self::Hold,
            "Jog" => Self::Jog,
            "Alarm" => Self::Alarm,
            "Door" => Self::Door,
            "Check" => Self::Check,
            "Home" => Self::Home,
            "Sleep" => Self::Sleep,
            _ => return None,
        })
    }

    /// Whether the machine is moving in this state.
    #[must_use]
    pub fn is_moving(self) -> bool {
        matches!(self, Self::Run | Self::Jog | Self::Home)
    }
}

impl fmt::Display for MachineState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_states_with_and_without_substates() {
        assert_eq!(MachineState::parse("Idle"), Some(MachineState::Idle));
        assert_eq!(MachineState::parse("Hold:1"), Some(MachineState::Hold));
        assert_eq!(MachineState::parse("Door:3"), Some(MachineState::Door));
        assert_eq!(MachineState::parse("idle"), None);
        assert_eq!(MachineState::parse(""), None);
    }

    #[test]
    fn display_round_trips_through_parse() {
        for state in [
            MachineState::Idle,
            MachineState::Run,
            MachineState::Hold,
            MachineState::Jog,
            MachineState::Alarm,
            MachineState::Door,
            MachineState::Check,
            MachineState::Home,
            MachineState::Sleep,
        ] {
            assert_eq!(MachineState::parse(&state.to_string()), Some(state));
        }
    }

    #[test]
    fn moving_states() {
        assert!(MachineState::Run.is_moving());
        assert!(!MachineState::Idle.is_moving());
        assert!(!MachineState::Alarm.is_moving());
    }
}

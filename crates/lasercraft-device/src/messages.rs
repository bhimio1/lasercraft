use std::fmt;

/// What the UI or CLI asks the device thread to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Open a serial port.
    Connect {
        /// Port name, such as `/dev/ttyUSB0` or `COM3`.
        port: String,
        /// Baud rate.
        baud: u32,
    },
    /// Stop streaming and close the port.
    Disconnect,
    /// Feed hold (`!`).
    Pause,
    /// Resume after a hold (`~`).
    Resume,
    /// Abort: stop streaming, then soft reset (0x18).
    Stop,
}

/// What the device thread reports back.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The port is open and the controller answered.
    Connected {
        /// Port name.
        port: String,
    },
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
        /// Machine position in millimetres, if reported.
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

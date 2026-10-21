//! Constants from GRBL v1.1's published interface documentation.

/// Real-time status report request. Not buffered; answered immediately.
pub const STATUS_QUERY: u8 = b'?';

/// Feed hold: decelerate to a stop and pause, keeping position.
pub const FEED_HOLD: u8 = b'!';

/// Cycle start / resume after a feed hold.
pub const CYCLE_START: u8 = b'~';

/// Soft reset (Ctrl-X): halt immediately and reset GRBL, which clears the
/// planner buffer. GRBL keeps its position unless it was moving, in which
/// case it raises an alarm because steps may have been lost.
pub const SOFT_RESET: u8 = 0x18;

/// Toggle spindle stop: switches the spindle (laser) off or back on, only
/// while a feed hold is complete. GRBL restores it on resume.
pub const SPINDLE_STOP: u8 = 0x9E;

/// The GRBL version banner starts with this.
pub const BANNER: &str = "Grbl ";

/// Longest line GRBL accepts (its line buffer).
pub const MAX_LINE: usize = 80;

/// Size of GRBL's serial receive buffer in bytes, used by character-counting
/// streaming.
pub const RX_BUFFER_SIZE: usize = 128;

/// The real-time bytes, which GRBL acts on as soon as they arrive, even in the
/// middle of a line.
pub const REALTIME: [u8; 5] = [
    STATUS_QUERY,
    FEED_HOLD,
    CYCLE_START,
    SOFT_RESET,
    SPINDLE_STOP,
];

/// Whether a byte is one of the real-time commands above.
#[must_use]
pub fn is_realtime(byte: u8) -> bool {
    REALTIME.contains(&byte)
}

/// What a GRBL alarm code means, in our words.
#[must_use]
pub fn alarm_text(code: u16) -> &'static str {
    match code {
        1 => "a limit switch was hit; the position is probably lost, home the machine",
        2 => "a move would go past the machine's travel (soft limit); the position is kept",
        3 => "reset while moving; the position is probably lost, home the machine",
        4 | 5 => "probe cycle failed",
        6 => "homing was reset before it finished",
        7 => "the safety door was opened while homing",
        8 => "homing could not move off the limit switch",
        9 => "homing did not find a limit switch",
        10 => "homing found only one of the two switches on a dual axis",
        _ => "unknown alarm",
    }
}

/// What a GRBL error code means, in our words.
#[must_use]
pub fn error_text(code: u16) -> &'static str {
    match code {
        1 => "a word has no letter",
        2 => "a number is missing or malformed",
        3 => "unknown $ command",
        4 => "a value must not be negative",
        5 => "homing is not enabled ($22)",
        6 => "step pulse must be at least 3 microseconds",
        7 => "settings could not be read and were reset to defaults",
        8 => "this $ command needs the machine to be idle",
        9 => "G-code is locked out during an alarm or a jog",
        10 => "soft limits need homing enabled",
        11 => "the line is too long",
        12 => "the setting would make the step rate too high",
        13 => "the safety door is open",
        14 => "the line is too long to store",
        15 => "the jog goes past the machine's travel",
        16 => "invalid jog command",
        17 => "laser mode needs PWM output",
        20 => "unsupported G-code command",
        21 => "two commands from the same group in one line",
        22 => "no feed rate set",
        23 => "the command needs a whole number",
        24 => "two commands in the line need axis words",
        25 => "a word is repeated in the line",
        26 => "the command needs axis words",
        27 => "invalid line number",
        28 => "the command is missing a value",
        29 => "G59.x coordinate systems are not supported",
        30 => "G53 needs G0 or G1",
        31 => "axis words without a command that uses them",
        32 => "arcs need an axis word in the plane",
        33 => "invalid motion target",
        34 => "invalid arc radius",
        35 => "arcs need an offset in the plane",
        36 => "unused words in the line",
        37 => "tool length offset is not on the configured axis",
        38 => "tool number too high",
        _ => "unknown error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realtime_bytes_are_recognized() {
        for b in [b'?', b'!', b'~', 0x18, 0x9E] {
            assert!(is_realtime(b));
        }
        assert!(!is_realtime(b'G'));
        assert!(!is_realtime(b'\n'));
    }

    #[test]
    fn codes_have_explanations() {
        assert!(alarm_text(1).contains("limit switch"));
        assert!(error_text(15).contains("travel"));
        assert_eq!(error_text(99), "unknown error");
        assert_eq!(alarm_text(99), "unknown alarm");
    }
}

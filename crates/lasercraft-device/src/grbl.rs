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

/// Size of GRBL's serial receive buffer in bytes, used by character-counting
/// streaming.
pub const RX_BUFFER_SIZE: usize = 128;

/// The real-time bytes, which GRBL acts on as soon as they arrive, even in the
/// middle of a line.
pub const REALTIME: [u8; 4] = [STATUS_QUERY, FEED_HOLD, CYCLE_START, SOFT_RESET];

/// Whether a byte is one of the real-time commands above.
#[must_use]
pub fn is_realtime(byte: u8) -> bool {
    REALTIME.contains(&byte)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realtime_bytes_are_recognized() {
        for b in [b'?', b'!', b'~', 0x18] {
            assert!(is_realtime(b));
        }
        assert!(!is_realtime(b'G'));
        assert!(!is_realtime(b'\n'));
    }
}

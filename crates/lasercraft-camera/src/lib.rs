//! Cameras for LaserCraft (`docs/spec/camera.md`): pictures, the lens
//! model and its calibration from pictures of the calibration sheet, the
//! bed alignment from burned marks, and the overlay that puts a picture on
//! the bed. Everything here is pure and deterministic; capturing pictures
//! happens in the app.

pub mod align;
pub mod calibrate;
pub mod controls;
pub mod dots;
pub mod homography;
pub mod lens;
mod linalg;
pub mod marks;
pub mod overlay;
pub mod picture;
pub mod scan;
pub mod sheet;
pub mod synth;

pub use homography::Homography;
pub use lens::{Lens, Model};
pub use picture::Picture;

/// Why a camera step failed, in words for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraError(pub String);

impl CameraError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for CameraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CameraError {}

/// The current time as RFC 3339 in UTC, seconds precision (for `created`).
#[must_use]
pub fn timestamp_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    timestamp(secs)
}

/// Seconds since 1970 as RFC 3339 in UTC.
#[must_use]
pub fn timestamp(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Civil date from days (H. Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem / 60 % 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn timestamps_are_rfc3339() {
        assert_eq!(super::timestamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(super::timestamp(1_792_627_200), "2026-10-22T00:00:00Z");
        assert_eq!(
            super::timestamp(951_782_400 + 86_399),
            "2000-02-29T23:59:59Z"
        );
    }
}

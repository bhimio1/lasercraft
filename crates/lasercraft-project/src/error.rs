use std::error::Error;
use std::fmt;

/// Why the project rejected a change or a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectError {
    /// A value is out of range or malformed.
    InvalidValue {
        /// What the value is, such as `work area width`.
        field: String,
        /// What is wrong with it.
        reason: String,
    },
    /// The file is not a LaserCraft project or is damaged.
    Format(String),
    /// The file was written by a newer LaserCraft.
    UnsupportedVersion(u32),
    /// Reading or writing the file failed.
    Io(String),
}

impl ProjectError {
    pub(crate) fn invalid(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidValue {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue { field, reason } => write!(f, "{field} {reason}"),
            Self::Format(reason) => write!(f, "not a valid LaserCraft project: {reason}"),
            Self::UnsupportedVersion(v) => write!(
                f,
                "project format version {v} is newer than this LaserCraft supports ({}); \
                 please update LaserCraft",
                crate::FORMAT_VERSION
            ),
            Self::Io(reason) => f.write_str(reason),
        }
    }
}

impl Error for ProjectError {}

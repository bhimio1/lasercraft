//! Built-in commands, one module per area.

mod project;

use crate::Command;

/// Every built-in command.
pub(crate) fn all() -> Vec<Box<dyn Command>> {
    project::commands()
}

//! The command registry: the only way to change a LaserCraft project.
//!
//! UI actions, keyboard shortcuts, the CLI and (later) an MCP server all
//! become an [`Invocation`] that a [`Session`] runs against a [`Registry`] of
//! [`Command`]s. Undoable commands return their inverse invocation
//! (`docs/spec/commands.md`).
//!
//! Conventions shared by every built-in command:
//! - Names are `area.action`, such as `project.set_work_area`.
//! - Lengths are millimetres, angles degrees.

mod action;
mod args;
mod cmds;
mod command;
mod error;
mod invocation;
mod registry;
mod session;

pub use action::action_name;
pub use cmds::device::{DeviceContext, with_context};
pub use command::{Command, Outcome};
pub use error::CommandError;
pub use invocation::Invocation;
pub use registry::Registry;
pub use session::{BATCH, REDO, SESSION_COMMANDS, Session, UNDO};

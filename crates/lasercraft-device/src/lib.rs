//! Laser controller connection for LaserCraft.
//!
//! The device runs on its own thread. The UI and CLI talk to it only through
//! two channels carrying the fixed message types [`Request`] and [`Event`]
//! (`docs/spec/device.md`), so a slow or stuck port can never block the UI.
//!
//! M0 defines the message types, GRBL's real-time bytes and state names. The
//! serial port, the streaming protocol and the simulated GRBL device arrive
//! with M6.

pub mod grbl;
mod messages;
pub mod permissions;

pub use messages::{Event, MachineState, Request};

//! Laser controller connection for LaserCraft (`docs/spec/device.md`).
//!
//! The device runs on its own thread ([`spawn`]). The UI and CLI talk to it
//! only through two channels carrying the fixed message types [`Request`]
//! and [`Event`], so a slow or stuck port can never block the UI.
//!
//! The streaming logic is the I/O-free [`Controller`]; [`sim::Sim`] is a
//! simulated GRBL controller, and [`Bench`] runs the two together on
//! simulated time for tests and the CLI.

mod bench;
mod controller;
pub mod grbl;
mod messages;
pub mod permissions;
pub mod reply;
pub mod sim;
mod thread;

pub use bench::Bench;
pub use controller::{Controller, Protocol, Settings, jog_line};
pub use messages::{Event, Info, MachineState, Outcome, Request, SIMULATOR_PORT, StreamKind};
pub use thread::{DeviceHandle, list_ports, spawn};

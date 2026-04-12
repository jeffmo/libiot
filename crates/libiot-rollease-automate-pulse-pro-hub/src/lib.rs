#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Async Rust client for the **Rollease Acmeda Automate Pulse Pro** shade
//! hub, speaking the local LAN ASCII protocol on TCP port 1487. No auth,
//! no cloud, no TLS.
//!
//! Part of the [`libiot`](https://github.com/jeffmo/libiot) workspace — a
//! collection of Rust connector libraries for consumer `IoT` devices.
//!
//! # Quick start
//!
//! Close every shade in the house using the broadcast address:
//!
//! ```no_run
//! use libiot_rollease_automate_pulse_pro_hub::AutomatePulseProHub;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let hub = AutomatePulseProHub::connect("192.168.5.234:1487").await?;
//! hub.close_all().await?;
//! # Ok(()) }
//! ```
//!
//! Move one specific shade to 50% closed by its hub-assigned address:
//!
//! ```no_run
//! use libiot_rollease_automate_pulse_pro_hub::AutomatePulseProHub;
//! use libiot_rollease_automate_pulse_pro_hub::MotorAddress;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let hub = AutomatePulseProHub::connect("192.168.5.234:1487").await?;
//! let kitchen = MotorAddress::new("3YC")?;
//! hub.set_position(&kitchen, 50).await?;
//! # Ok(()) }
//! ```
//!
//! Query the full hub info — hub name, serial, and every paired
//! motor's current position and friendly name — in a single batched
//! round-trip:
//!
//! ```no_run
//! use libiot_rollease_automate_pulse_pro_hub::AutomatePulseProHub;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let hub = AutomatePulseProHub::connect("192.168.5.234:1487").await?;
//! let info = hub.info().await?;
//! println!("hub: {} (serial {})", info.hub_name, info.hub_serial);
//! for motor in &info.motors {
//!     println!("  {}: {:?}", motor.address, motor.position);
//! }
//! # Ok(()) }
//! ```
//!
//! # References
//!
//! The primary protocol reference for this crate is the in-crate copy of
//! `PULSE_PRO_LOCAL_API.md` at the crate root, which combines the
//! official Rollease Acmeda *Automate Pulse 2 LINQ* integrator PDF, the
//! open-source [`aiopulse2`](https://github.com/sillyfrog/aiopulse2)
//! Python library, and direct probing of a real hub. External
//! references worth keeping handy:
//!
//! - [`aiopulse2`](https://github.com/sillyfrog/aiopulse2) — reference
//!   implementation for both the ASCII channel (what this crate uses)
//!   and the undocumented WebSocket RPC channel (tracked as future
//!   work — see the crate's `project-tracker.md`).
//! - The Rollease Acmeda *Automate Pulse 2 LINQ* integrator PDF — the
//!   canonical but incomplete ASCII spec. Several field-verified
//!   deviations from the PDF are documented inline in this crate's
//!   codec module.
//!
//! # Feature flags
//!
//! - `dangerous-ops` — opts in to destructive protocol operations
//!   (pair motor, unpair, delete, factory reset, set/delete limits).
//!   Off by default: a typo here can wipe the hub's pairing
//!   configuration.

mod automate_pulse_pro_hub;
mod codec;
mod error;
mod hub_info;
mod motor;
mod motor_address;
mod motor_position;
mod motor_type;
mod motor_version;
mod motor_voltage;
mod transport;

#[cfg(test)]
mod tests;

pub use crate::automate_pulse_pro_hub::AutomatePulseProHub;
pub use crate::automate_pulse_pro_hub::DEFAULT_PORT;
pub use crate::error::Error;
pub use crate::error::HubErrorCode;
pub use crate::error::Result;
pub use crate::hub_info::HubInfo;
pub use crate::motor::Motor;
pub use crate::motor_address::MotorAddress;
pub use crate::motor_position::MotorPosition;
pub use crate::motor_type::MotorType;
pub use crate::motor_version::MotorVersion;
pub use crate::motor_voltage::MotorVoltage;

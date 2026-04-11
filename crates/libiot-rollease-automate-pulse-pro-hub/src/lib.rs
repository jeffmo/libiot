//! Async Rust client for the Rollease Acmeda Automate Pulse Pro shade
//! hub, speaking the local LAN ASCII protocol on TCP port 1487. No
//! auth, no cloud, no TLS.
//!
//! Part of the [`libiot`](https://github.com/jeffmo/libiot) workspace.
//!
//! This commit lands the generic async transport layer
//! (`Transport<S>`) on top of the pure codec from the previous
//! commit. The public `AutomatePulseProHub` client lands in the next
//! commit, at which point the crate-level rustdoc is rewritten with
//! real usage examples and a References section.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod error;
mod hub_snapshot;
mod motor;
mod motor_address;
mod motor_position;
mod motor_type;
mod motor_version;
mod motor_voltage;
mod transport;

#[cfg(test)]
mod tests;

pub use crate::error::Error;
pub use crate::error::HubErrorCode;
pub use crate::error::Result;
pub use crate::hub_snapshot::HubSnapshot;
pub use crate::motor::Motor;
pub use crate::motor_address::MotorAddress;
pub use crate::motor_position::MotorPosition;
pub use crate::motor_type::MotorType;
pub use crate::motor_version::MotorVersion;
pub use crate::motor_voltage::MotorVoltage;

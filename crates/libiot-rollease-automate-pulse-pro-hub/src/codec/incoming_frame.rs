//! The [`IncomingFrame`] enum — one variant per frame shape the hub emits.

use crate::error::HubErrorCode;
use crate::motor_address::MotorAddress;
use crate::motor_position::MotorPosition;
use crate::motor_type::MotorType;
use crate::motor_voltage::MotorVoltage;

/// A single parsed response frame from the Pulse Pro hub.
///
/// The variants cover every distinct frame shape documented in the
/// in-crate `PULSE_PRO_LOCAL_API.md`. The parser in
/// [`crate::codec::parse_frames`] produces values of this type; the
/// transport layer forwards them up to the client layer, which
/// matches on the variant and returns the appropriate typed value.
///
/// Variants are sorted alphabetically per the workspace code-style
/// rules (iot-crate-standards §6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum IncomingFrame {
    /// A pair/unpair/rename acknowledgement (`!<addr>A;`).
    AddressAck { addr: MotorAddress },

    /// A typed error response (`!<addr>E<xx>;`).
    HubError {
        addr: MotorAddress,
        code: HubErrorCode,
    },

    /// The hub's friendly name (`!000NAME<string>;`).
    HubName(String),

    /// The hub's serial number (`!000SN<string>;`).
    HubSerial(String),

    /// A motor's friendly name (`!<addr>NAME<string>;`).
    MotorName { addr: MotorAddress, name: String },

    /// A motor position reply (`!<addr>r<closed3>b<tilt3>,R<sig2>;`).
    MotorPositionRec {
        addr: MotorAddress,
        position: MotorPosition,
    },

    /// A motor or hub version record (`!<addr>v<type><ver>;`).
    ///
    /// The hub emits these in response to either the broadcast
    /// enumeration query (`!000v?;`) — which produces one record per
    /// paired device plus one for the hub itself — or a per-motor
    /// version query.
    MotorVersionRec {
        addr: MotorAddress,
        motor_type: MotorType,
        version: String,
    },

    /// A motor voltage reply (`!<addr>pVc<NNNNN>,R<sig2>;`).
    MotorVoltageRec {
        addr: MotorAddress,
        voltage: MotorVoltage,
    },
}

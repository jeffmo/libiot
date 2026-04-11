//! The [`MotorVersion`] struct — parsed motor version reply.

use crate::motor_address::MotorAddress;
use crate::motor_type::MotorType;

/// A parsed version-query reply identifying one paired device.
///
/// The hub emits a `MotorVersion` in response to either a per-motor
/// version query (`!<addr>v?;`) or the broadcast enumeration query
/// (`!000v?;`). The broadcast form produces one `MotorVersion` per
/// paired device plus one for the hub itself — callers that want a
/// list of motors only should filter out entries whose `motor_type`
/// is [`MotorType::HubGateway`].
///
/// Wire format:
///
/// ```text
/// !<addr>v<type><ver>;
/// ```
///
/// where `<type>` is one ASCII byte and `<ver>` is a variable-length
/// version string, typically 2-3 digits (e.g. `22` → v2.2).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MotorVersion {
    /// The 3-character address of the device this version record
    /// describes.
    pub address: MotorAddress,

    /// The motor's hardware class. [`MotorType::HubGateway`] means this
    /// record describes the hub itself, not a paired motor.
    pub motor_type: MotorType,

    /// The motor's firmware version string exactly as reported by the
    /// hub. Typically a 2-3 digit numeric string (e.g. `"22"` or `"10"`),
    /// but recorded verbatim so unusual values round-trip cleanly.
    pub version: String,
}

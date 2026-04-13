//! The [`Motor`] struct — aggregated description of one paired motor.

use crate::motor_address::MotorAddress;
use crate::motor_position::MotorPosition;
use crate::motor_version::MotorVersion;
use crate::motor_voltage::MotorVoltage;

/// Aggregated description of one paired motor, as assembled by
/// [`crate::AutomatePulseProHub::info`] or
/// [`crate::AutomatePulseProHub::list_motors`].
///
/// A `Motor` value represents the client's best available view of a
/// single paired motor at a single point in time. The `name` and
/// `position` fields are both `Option` because the hub may fail to
/// report them independently:
///
/// - `name` is `None` if the user hasn't assigned a friendly name to
///   the motor in the Pulse 2 mobile app, or if the name query failed.
/// - `position` is `None` when the motor was discovered through the
///   version enumeration query but did not respond to the subsequent
///   position query — typically because the motor is offline, out of
///   RF range, or has a dead battery.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Motor {
    /// The hub-assigned 3-character address that identifies this motor
    /// on the wire.
    pub address: MotorAddress,

    /// The motor's friendly name as set in the Pulse 2 mobile app. `None`
    /// means the name was either not set or could not be retrieved.
    pub name: Option<String>,

    /// The motor's firmware/type record from the broadcast enumeration
    /// query.
    pub version: MotorVersion,

    /// The motor's current lift/tilt position. `None` means the motor was
    /// discovered but did not reply to the position query (typically
    /// offline or out of range).
    pub position: Option<MotorPosition>,

    /// The motor's battery voltage. `None` means the motor did not reply
    /// to the voltage query (typically offline, out of range, or an AC
    /// motor that does not report battery level).
    pub voltage: Option<MotorVoltage>,
}

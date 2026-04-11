//! Pure frame-encoding free functions.
//!
//! Each `encode_*` function returns the exact on-wire bytes for one
//! Pulse Pro ASCII command. Nothing here does I/O or async — these
//! functions are directly unit-testable without any runtime setup.
//!
//! Every encoder's output is covered by at least one test in
//! `crate::codec::tests::encoder_tests`.

use crate::error::Error;
use crate::error::Result;
use crate::motor_address::MotorAddress;

/// The broadcast address that targets every paired motor on the hub.
const BROADCAST_BYTES: &[u8; 3] = b"000";

/// Encode an "open" command (`!<addr>o;`) — lift the motor to 0% closed.
pub(crate) fn encode_open(addr: &MotorAddress) -> Vec<u8> {
    build_simple_frame(addr.as_bytes(), b"o")
}

/// Encode a "close" command (`!<addr>c;`) — lift the motor to 100% closed.
pub(crate) fn encode_close(addr: &MotorAddress) -> Vec<u8> {
    build_simple_frame(addr.as_bytes(), b"c")
}

/// Encode a "stop" command (`!<addr>s;`) — halt any in-flight motion.
pub(crate) fn encode_stop(addr: &MotorAddress) -> Vec<u8> {
    build_simple_frame(addr.as_bytes(), b"s")
}

/// Encode a broadcast "open" command (`!000o;`) — target every motor.
pub(crate) fn encode_open_all() -> Vec<u8> {
    build_simple_frame(BROADCAST_BYTES, b"o")
}

/// Encode a broadcast "close" command (`!000c;`).
pub(crate) fn encode_close_all() -> Vec<u8> {
    build_simple_frame(BROADCAST_BYTES, b"c")
}

/// Encode a broadcast "stop" command (`!000s;`).
pub(crate) fn encode_stop_all() -> Vec<u8> {
    build_simple_frame(BROADCAST_BYTES, b"s")
}

/// Encode a "jog open" command (`!<addr>oA;`) — nudge the motor one step up.
pub(crate) fn encode_jog_open(addr: &MotorAddress) -> Vec<u8> {
    build_simple_frame(addr.as_bytes(), b"oA")
}

/// Encode a "jog close" command (`!<addr>cA;`) — nudge one step down.
pub(crate) fn encode_jog_close(addr: &MotorAddress) -> Vec<u8> {
    build_simple_frame(addr.as_bytes(), b"cA")
}

/// Encode a "move to position" command (`!<addr>m<NNN>;`).
///
/// The `percent` argument is the desired closed-lift percentage and must
/// be in `0..=100`. Returns [`Error::InvalidPercentage`] otherwise.
///
/// Note the 3-digit padding — the vendor PDF documents 2 digits, but
/// the field-verified on-wire format is 3 digits. See §2.5 of the
/// in-crate `PULSE_PRO_LOCAL_API.md`.
pub(crate) fn encode_move_to(addr: &MotorAddress, percent: u8) -> Result<Vec<u8>> {
    if percent > 100 {
        return Err(Error::InvalidPercentage {
            value: u16::from(percent),
        });
    }
    Ok(build_three_digit_frame(addr.as_bytes(), b'm', percent))
}

/// Encode a "tilt" command (`!<addr>b<NNN>;`).
///
/// Same 3-digit padding and range-check as [`encode_move_to`].
pub(crate) fn encode_tilt(addr: &MotorAddress, percent: u8) -> Result<Vec<u8>> {
    if percent > 100 {
        return Err(Error::InvalidPercentage {
            value: u16::from(percent),
        });
    }
    Ok(build_three_digit_frame(addr.as_bytes(), b'b', percent))
}

/// Encode a query for the hub's friendly name (`!000NAME?;`).
pub(crate) fn encode_query_hub_name() -> Vec<u8> {
    b"!000NAME?;".to_vec()
}

/// Encode a query for the hub's serial number (`!000SN?;`).
pub(crate) fn encode_query_hub_serial() -> Vec<u8> {
    b"!000SN?;".to_vec()
}

/// Encode a broadcast motor enumeration query (`!000v?;`).
pub(crate) fn encode_query_motor_enum() -> Vec<u8> {
    b"!000v?;".to_vec()
}

/// Encode a query for a specific motor's version (`!<addr>v?;`).
pub(crate) fn encode_query_motor_version(addr: &MotorAddress) -> Vec<u8> {
    build_query_frame(addr.as_bytes(), b"v")
}

/// Encode a query for a specific motor's friendly name (`!<addr>NAME?;`).
pub(crate) fn encode_query_motor_name(addr: &MotorAddress) -> Vec<u8> {
    build_query_frame(addr.as_bytes(), b"NAME")
}

/// Encode a query for a specific motor's current position (`!<addr>r?;`).
pub(crate) fn encode_query_motor_position(addr: &MotorAddress) -> Vec<u8> {
    build_query_frame(addr.as_bytes(), b"r")
}

/// Encode a broadcast position query (`!000r?;`) — returns one position
/// frame per paired motor (motors that are offline or out of RF range
/// are silently omitted by the hub).
pub(crate) fn encode_query_motor_position_all() -> Vec<u8> {
    b"!000r?;".to_vec()
}

/// Encode a query for a specific motor's battery voltage (`!<addr>pVc?;`).
pub(crate) fn encode_query_motor_voltage(addr: &MotorAddress) -> Vec<u8> {
    build_query_frame(addr.as_bytes(), b"pVc")
}

// -- dangerous-ops --------------------------------------------------------
//
// All of the following commands are destructive: a typo here can wipe the
// user's hub pairing configuration. They're gated behind the
// `dangerous-ops` feature so callers must opt in at compile time.

/// Encode a "pair a new motor" command (`!000&;`). The hub auto-assigns
/// a random 3-char address to the newly paired motor.
#[cfg(feature = "dangerous-ops")]
pub(crate) fn encode_pair_motor() -> Vec<u8> {
    b"!000&;".to_vec()
}

/// Encode an "unpair motor" command (`!<addr>#;`). Requires the motor
/// to acknowledge the unpair.
#[cfg(feature = "dangerous-ops")]
pub(crate) fn encode_unpair_motor(addr: &MotorAddress) -> Vec<u8> {
    build_simple_frame(addr.as_bytes(), b"#")
}

/// Encode a "delete motor from hub" command (`!<addr>$;`). No motor
/// acknowledgement required — removes the address from the hub's
/// paired list.
#[cfg(feature = "dangerous-ops")]
pub(crate) fn encode_delete_motor(addr: &MotorAddress) -> Vec<u8> {
    build_simple_frame(addr.as_bytes(), b"$")
}

/// Encode a factory-reset command (`!000*;`) — wipes all hub
/// configuration.
#[cfg(feature = "dangerous-ops")]
pub(crate) fn encode_factory_reset() -> Vec<u8> {
    b"!000*;".to_vec()
}

/// Encode a "set upper limit at current position" command (`!<addr>pEoH;`).
#[cfg(feature = "dangerous-ops")]
pub(crate) fn encode_set_upper_limit(addr: &MotorAddress) -> Vec<u8> {
    build_simple_frame(addr.as_bytes(), b"pEoH")
}

/// Encode a "set lower limit at current position" command (`!<addr>pEcH;`).
#[cfg(feature = "dangerous-ops")]
pub(crate) fn encode_set_lower_limit(addr: &MotorAddress) -> Vec<u8> {
    build_simple_frame(addr.as_bytes(), b"pEcH")
}

/// Encode a "delete all limits" command (`!<addr>pEaC;`).
#[cfg(feature = "dangerous-ops")]
pub(crate) fn encode_delete_limits(addr: &MotorAddress) -> Vec<u8> {
    build_simple_frame(addr.as_bytes(), b"pEaC")
}

// -- internal helpers -----------------------------------------------------

/// Build a simple `!<addr><cmd>;` frame with no data payload.
fn build_simple_frame(addr: &[u8; 3], cmd: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 + addr.len() + cmd.len() + 1);
    out.push(b'!');
    out.extend_from_slice(addr);
    out.extend_from_slice(cmd);
    out.push(b';');
    out
}

/// Build a query frame `!<addr><cmd>?;`.
fn build_query_frame(addr: &[u8; 3], cmd: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + addr.len() + cmd.len() + 1);
    out.push(b'!');
    out.extend_from_slice(addr);
    out.extend_from_slice(cmd);
    out.push(b'?');
    out.push(b';');
    out
}

/// Build a 3-digit-padded frame `!<addr><cmd><NNN>;`.
fn build_three_digit_frame(addr: &[u8; 3], cmd: u8, value: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 + addr.len() + 1 + 3 + 1);
    out.push(b'!');
    out.extend_from_slice(addr);
    out.push(cmd);
    // 3-digit zero-padded decimal. Clippy would otherwise suggest write!,
    // but we already have a Vec<u8> and want to avoid an extra allocation.
    out.push(b'0' + (value / 100));
    out.push(b'0' + ((value / 10) % 10));
    out.push(b'0' + (value % 10));
    out.push(b';');
    out
}

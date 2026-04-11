//! Exhaustive tests for the free functions in [`crate::codec::encoder`].
//!
//! Every encoder is covered by at least one happy-path test that asserts
//! the exact output bytes. Encoders that take a percentage argument are
//! also tested at the edges of the valid range and with an out-of-range
//! value to verify they return [`crate::Error::InvalidPercentage`].

use crate::codec::encoder::encode_close;
use crate::codec::encoder::encode_close_all;
use crate::codec::encoder::encode_jog_close;
use crate::codec::encoder::encode_jog_open;
use crate::codec::encoder::encode_move_to;
use crate::codec::encoder::encode_open;
use crate::codec::encoder::encode_open_all;
use crate::codec::encoder::encode_query_hub_name;
use crate::codec::encoder::encode_query_hub_serial;
use crate::codec::encoder::encode_query_motor_enum;
use crate::codec::encoder::encode_query_motor_name;
use crate::codec::encoder::encode_query_motor_position;
use crate::codec::encoder::encode_query_motor_position_all;
use crate::codec::encoder::encode_query_motor_version;
use crate::codec::encoder::encode_query_motor_voltage;
use crate::codec::encoder::encode_stop;
use crate::codec::encoder::encode_stop_all;
use crate::codec::encoder::encode_tilt;
use crate::error::Error;
use crate::motor_address::MotorAddress;

// -- motor control ----------------------------------------------------------

/// Verifies `encode_open` produces `!<addr>o;` exactly, matching the
/// wire format documented in §2.2 of `PULSE_PRO_LOCAL_API.md`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn encode_open_produces_exact_wire_bytes() {
    let addr = MotorAddress::new("4JK").unwrap();
    assert_eq!(encode_open(&addr), b"!4JKo;");
}

/// Verifies `encode_close` produces `!<addr>c;` exactly. Matches the
/// downlink command documented in §2.2 of `PULSE_PRO_LOCAL_API.md`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn encode_close_produces_exact_wire_bytes() {
    let addr = MotorAddress::new("MWX").unwrap();
    assert_eq!(encode_close(&addr), b"!MWXc;");
}

/// Verifies `encode_stop` produces `!<addr>s;` exactly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn encode_stop_produces_exact_wire_bytes() {
    let addr = MotorAddress::new("3YC").unwrap();
    assert_eq!(encode_stop(&addr), b"!3YCs;");
}

/// Verifies the broadcast motor-control encoders target the reserved
/// address `000`, which the hub treats as "every paired motor".
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn broadcast_motor_control_encoders_use_000_address() {
    assert_eq!(encode_open_all(), b"!000o;");
    assert_eq!(encode_close_all(), b"!000c;");
    assert_eq!(encode_stop_all(), b"!000s;");
}

/// Verifies `encode_jog_open` produces `!<addr>oA;` exactly — the
/// 2-byte `oA` command nudges the motor one step up.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn encode_jog_open_produces_two_byte_command() {
    let addr = MotorAddress::new("4JK").unwrap();
    assert_eq!(encode_jog_open(&addr), b"!4JKoA;");
}

/// Verifies `encode_jog_close` produces `!<addr>cA;` exactly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn encode_jog_close_produces_two_byte_command() {
    let addr = MotorAddress::new("4JK").unwrap();
    assert_eq!(encode_jog_close(&addr), b"!4JKcA;");
}

// -- move_to and tilt (3-digit padding quirk) -----------------------------

/// Verifies `encode_move_to` pads the percentage to **3 digits**, not
/// 2. The vendor PDF documents a 2-digit format but the field-verified
/// on-wire format is 3 digits — see §2.5 of `PULSE_PRO_LOCAL_API.md`.
/// This test pins that behavior so it cannot silently regress.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn encode_move_to_uses_three_digit_padding_not_two() {
    let addr = MotorAddress::new("MWX").unwrap();
    assert_eq!(
        encode_move_to(&addr, /* percent = */ 50).unwrap(),
        b"!MWXm050;",
    );
    assert_eq!(
        encode_move_to(&addr, /* percent = */ 0).unwrap(),
        b"!MWXm000;",
    );
    assert_eq!(
        encode_move_to(&addr, /* percent = */ 100).unwrap(),
        b"!MWXm100;",
    );
    assert_eq!(
        encode_move_to(&addr, /* percent = */ 7).unwrap(),
        b"!MWXm007;",
    );
}

/// Verifies `encode_move_to` returns
/// [`Error::InvalidPercentage`] for values above 100.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn encode_move_to_rejects_out_of_range_percentage() {
    let addr = MotorAddress::new("MWX").unwrap();
    match encode_move_to(&addr, /* percent = */ 101) {
        Err(Error::InvalidPercentage { value }) => assert_eq!(value, 101),
        other => panic!("expected InvalidPercentage, got {other:?}"),
    }
    match encode_move_to(&addr, /* percent = */ 255) {
        Err(Error::InvalidPercentage { value }) => assert_eq!(value, 255),
        other => panic!("expected InvalidPercentage, got {other:?}"),
    }
}

/// Verifies `encode_tilt` uses the same 3-digit padding as
/// `encode_move_to` but with a different command byte (`b` not `m`).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn encode_tilt_uses_three_digit_padding_with_b_command() {
    let addr = MotorAddress::new("MWX").unwrap();
    assert_eq!(
        encode_tilt(&addr, /* percent = */ 50).unwrap(),
        b"!MWXb050;",
    );
    assert_eq!(encode_tilt(&addr, /* percent = */ 0).unwrap(), b"!MWXb000;",);
    assert_eq!(
        encode_tilt(&addr, /* percent = */ 100).unwrap(),
        b"!MWXb100;",
    );
}

/// Verifies `encode_tilt` returns
/// [`Error::InvalidPercentage`] for values above 100.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn encode_tilt_rejects_out_of_range_percentage() {
    let addr = MotorAddress::new("4JK").unwrap();
    match encode_tilt(&addr, /* percent = */ 250) {
        Err(Error::InvalidPercentage { value }) => assert_eq!(value, 250),
        other => panic!("expected InvalidPercentage, got {other:?}"),
    }
}

// -- queries ---------------------------------------------------------------

/// Verifies hub-level query encoders match the exact byte strings
/// documented in §2.3 of `PULSE_PRO_LOCAL_API.md`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn hub_level_queries_match_documented_wire_format() {
    assert_eq!(encode_query_hub_name(), b"!000NAME?;");
    assert_eq!(encode_query_hub_serial(), b"!000SN?;");
    assert_eq!(encode_query_motor_enum(), b"!000v?;");
    assert_eq!(encode_query_motor_position_all(), b"!000r?;");
}

/// Verifies per-motor query encoders produce `!<addr><cmd>?;` frames
/// with the correct command sequences for every query we support.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn per_motor_queries_match_documented_wire_format() {
    let addr = MotorAddress::new("4JK").unwrap();
    assert_eq!(encode_query_motor_name(&addr), b"!4JKNAME?;");
    assert_eq!(encode_query_motor_version(&addr), b"!4JKv?;");
    assert_eq!(encode_query_motor_position(&addr), b"!4JKr?;");
    assert_eq!(encode_query_motor_voltage(&addr), b"!4JKpVc?;");
}

// -- dangerous-ops ---------------------------------------------------------

/// Verifies destructive-operation encoders match the documented
/// pairing / reset wire format from §2.6 of `PULSE_PRO_LOCAL_API.md`.
/// Only compiled under the `dangerous-ops` feature.
///
/// Written by Claude Code, reviewed by a human.
#[cfg(feature = "dangerous-ops")]
#[test]
fn dangerous_ops_encoders_match_documented_wire_format() {
    use crate::codec::encoder::encode_delete_limits;
    use crate::codec::encoder::encode_delete_motor;
    use crate::codec::encoder::encode_factory_reset;
    use crate::codec::encoder::encode_pair_motor;
    use crate::codec::encoder::encode_set_lower_limit;
    use crate::codec::encoder::encode_set_upper_limit;
    use crate::codec::encoder::encode_unpair_motor;

    let addr = MotorAddress::new("4JK").unwrap();

    assert_eq!(encode_pair_motor(), b"!000&;");
    assert_eq!(encode_unpair_motor(&addr), b"!4JK#;");
    assert_eq!(encode_delete_motor(&addr), b"!4JK$;");
    assert_eq!(encode_factory_reset(), b"!000*;");
    assert_eq!(encode_set_upper_limit(&addr), b"!4JKpEoH;");
    assert_eq!(encode_set_lower_limit(&addr), b"!4JKpEcH;");
    assert_eq!(encode_delete_limits(&addr), b"!4JKpEaC;");
}

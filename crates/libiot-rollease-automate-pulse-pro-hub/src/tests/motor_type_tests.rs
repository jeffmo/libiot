//! Tests for [`crate::MotorType`] — wire byte mapping and hub-gateway
//! detection.

use crate::motor_type::MotorType;

/// Verifies every wire byte documented in §2.4 of
/// `PULSE_PRO_LOCAL_API.md` maps to the correct [`MotorType`] variant,
/// and that the byte → variant round-trip is lossless.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_type_round_trips_every_documented_wire_byte() {
    let cases: &[(u8, MotorType)] = &[
        (b'A', MotorType::Ac),
        (b'B', MotorType::HubGateway),
        (b'C', MotorType::Curtain),
        (b'D', MotorType::Dc),
        (b'd', MotorType::DcLower),
        (b'L', MotorType::Light),
        (b'S', MotorType::Socket),
        (b'U', MotorType::DcU),
    ];

    for (byte, expected) in cases {
        assert_eq!(
            MotorType::from_wire_byte(*byte),
            Some(*expected),
            "byte = {:?}",
            *byte as char,
        );
        assert_eq!(expected.wire_byte(), *byte);
    }
}

/// Verifies that bytes outside the documented alphabet return `None`
/// from `MotorType::from_wire_byte` — the parser treats `None` as a
/// malformed-frame error rather than silently dropping the frame.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_type_from_wire_byte_returns_none_for_unknown_bytes() {
    for bad in [b'X', b'Y', b'Z', b'0', b'!', b' ', b'\n'] {
        assert_eq!(
            MotorType::from_wire_byte(bad),
            None,
            "byte = {:?}",
            bad as char
        );
    }
}

/// Verifies `is_hub_gateway` is `true` only for the `HubGateway`
/// variant and `false` for every actual motor variant.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_type_is_hub_gateway_only_matches_hub_gateway_variant() {
    assert!(MotorType::HubGateway.is_hub_gateway());

    for variant in [
        MotorType::Ac,
        MotorType::Curtain,
        MotorType::Dc,
        MotorType::DcLower,
        MotorType::DcU,
        MotorType::Light,
        MotorType::Socket,
    ] {
        assert!(!variant.is_hub_gateway(), "variant = {variant:?}");
    }
}

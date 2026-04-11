//! Tests for [`crate::Error`] and [`crate::HubErrorCode`] — mapping
//! from wire-format strings and error message formatting.

use crate::error::Error;
use crate::error::HubErrorCode;
use crate::motor_address::MotorAddress;

/// Verifies that every documented hub error code string from §2.8 of
/// `PULSE_PRO_LOCAL_API.md` maps to the corresponding `HubErrorCode`
/// variant via `from_wire`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn hub_error_code_from_wire_maps_every_documented_code() {
    let cases: &[(&str, HubErrorCode)] = &[
        ("bz", HubErrorCode::Busy),
        ("df", HubErrorCode::TooManyMotors),
        ("np", HubErrorCode::NoSuchMotor),
        ("nc", HubErrorCode::NoLimitsSet),
        ("mh", HubErrorCode::MasterHallSensor),
        ("sh", HubErrorCode::SlaveHallSensor),
        ("or", HubErrorCode::ObstacleUp),
        ("cr", HubErrorCode::ObstacleDown),
        ("pl", HubErrorCode::LowVoltage),
        ("ph", HubErrorCode::HighVoltage),
        ("nl", HubErrorCode::MotorOffline),
        ("ec", HubErrorCode::Generic),
    ];

    for (wire, expected) in cases {
        assert_eq!(HubErrorCode::from_wire(wire), *expected, "wire = {wire:?}");
    }
}

/// Verifies that `HubErrorCode::from_wire` captures unknown codes as
/// `Unknown(code)` rather than dropping them. This is important for
/// forward-compatibility with future hub firmware that may introduce
/// new error codes.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn hub_error_code_from_wire_captures_unknown_codes() {
    assert_eq!(
        HubErrorCode::from_wire("zz"),
        HubErrorCode::Unknown("zz".into()),
    );
    assert_eq!(
        HubErrorCode::from_wire("q1"),
        HubErrorCode::Unknown("q1".into()),
    );
}

/// Verifies that `Error` variants produce descriptive `Display`
/// messages suitable for logging, including the context fields on
/// each variant.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn error_display_includes_context_fields() {
    let invalid = Error::InvalidAddress {
        input: "abcd".into(),
    };
    assert!(format!("{invalid}").contains("abcd"));

    let invalid_pct = Error::InvalidPercentage { value: 250 };
    assert!(format!("{invalid_pct}").contains("250"));

    let hub_err = Error::HubError {
        address: MotorAddress::new("4JK").unwrap(),
        code: HubErrorCode::MotorOffline,
    };
    let rendered = format!("{hub_err}");
    assert!(rendered.contains("4JK"));
    assert!(rendered.contains("MotorOffline"));

    let timeout = Error::Timeout { ms: 1500 };
    assert!(format!("{timeout}").contains("1500"));

    let malformed = Error::Malformed {
        detail: "bad frame".into(),
        raw: "!!!".into(),
    };
    let rendered = format!("{malformed}");
    assert!(rendered.contains("bad frame"));
    assert!(rendered.contains("!!!"));
}

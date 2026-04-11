//! Tests for [`crate::MotorAddress`] construction, display, and
//! case-sensitivity invariants.

use std::str::FromStr;

use crate::error::Error;
use crate::motor_address::MotorAddress;

/// Verifies `MotorAddress::new` accepts every 3-char alphanumeric
/// input documented in §2.1 of `PULSE_PRO_LOCAL_API.md` — digits,
/// uppercase letters, lowercase letters, and mixed combinations.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_address_new_accepts_all_alphanumeric_3char_inputs() {
    // Digits only.
    assert!(MotorAddress::new("000").is_ok());
    assert!(MotorAddress::new("123").is_ok());
    // Letters only.
    assert!(MotorAddress::new("ABC").is_ok());
    assert!(MotorAddress::new("xyz").is_ok());
    // Mixed.
    assert!(MotorAddress::new("4JK").is_ok());
    assert!(MotorAddress::new("MWX").is_ok());
    assert!(MotorAddress::new("3YC").is_ok());
    assert!(MotorAddress::new("BR1").is_ok());
}

/// Verifies `MotorAddress::new` rejects inputs that are the wrong
/// length, returning `Error::InvalidAddress`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_address_new_rejects_wrong_length_inputs() {
    for bad in ["", "A", "AB", "ABCD", "ABCDE"] {
        match MotorAddress::new(bad) {
            Err(Error::InvalidAddress { input }) => assert_eq!(input, bad),
            other => panic!("expected InvalidAddress for {bad:?}, got {other:?}"),
        }
    }
}

/// Verifies `MotorAddress::new` rejects inputs containing
/// non-alphanumeric bytes (e.g. symbols, whitespace, non-ASCII chars).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_address_new_rejects_non_alphanumeric_bytes() {
    for bad in ["4J!", "4 K", "4-K", "4J\n", "4Jé"] {
        match MotorAddress::new(bad) {
            Err(Error::InvalidAddress { .. }) => {},
            other => panic!("expected InvalidAddress for {bad:?}, got {other:?}"),
        }
    }
}

/// Verifies that the hub's documented case-sensitivity is preserved
/// — `"4jk"` and `"4JK"` are distinct addresses, as §2.9 quirk #3 of
/// `PULSE_PRO_LOCAL_API.md` requires.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_address_preserves_case_sensitivity() {
    let upper = MotorAddress::new("4JK").unwrap();
    let lower = MotorAddress::new("4jk").unwrap();
    assert_ne!(upper, lower);
    assert_eq!(upper.as_str(), "4JK");
    assert_eq!(lower.as_str(), "4jk");
}

/// Verifies the broadcast address constant equals `"000"` and is
/// correctly detected by `is_broadcast`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_address_broadcast_constant_and_detection() {
    assert_eq!(MotorAddress::BROADCAST.as_str(), "000");
    assert!(MotorAddress::BROADCAST.is_broadcast());

    // Non-broadcast address.
    let non = MotorAddress::new("4JK").unwrap();
    assert!(!non.is_broadcast());
}

/// Verifies that `Display` and `Debug` for `MotorAddress` produce
/// useful representations without leaking raw bytes.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_address_display_and_debug_formatting() {
    let addr = MotorAddress::new("MWX").unwrap();
    assert_eq!(format!("{addr}"), "MWX");
    assert_eq!(format!("{addr:?}"), "MotorAddress(\"MWX\")");
}

/// Verifies that `FromStr` and `TryFrom<&str>` delegate to
/// `MotorAddress::new` so the validation rules apply uniformly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_address_from_str_and_try_from_share_new_validation() {
    let ok: MotorAddress = "4JK".parse().unwrap();
    assert_eq!(ok.as_str(), "4JK");

    let ok2: MotorAddress = MotorAddress::try_from("MWX").unwrap();
    assert_eq!(ok2.as_str(), "MWX");

    assert!(MotorAddress::from_str("too-long").is_err());
    assert!(MotorAddress::try_from("!!").is_err());
}

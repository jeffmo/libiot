//! Tests for [`crate::motor_selector`] — `MotorSelector` parsing and
//! name resolution against a hand-crafted `HubInfo`.

use libiot_rollease_automate_pulse_pro_hub::HubInfo;
use libiot_rollease_automate_pulse_pro_hub::Motor;
use libiot_rollease_automate_pulse_pro_hub::MotorAddress;
use libiot_rollease_automate_pulse_pro_hub::MotorType;
use libiot_rollease_automate_pulse_pro_hub::MotorVersion;

use crate::error::CliError;
use crate::motor_selector::MotorSelector;
use crate::motor_selector::resolve_against_hub_info;

fn addr(s: &str) -> MotorAddress {
    MotorAddress::new(s).unwrap()
}

/// Build a test `HubInfo` with three motors: 4JK "John House",
/// MWX "Dining Room", 3YC "Kitchen". Mirrors the real hub captures
/// from `PULSE_PRO_LOCAL_API.md` §6.
fn test_hub_info() -> HubInfo {
    HubInfo {
        hub_name: "Test Hub".to_owned(),
        hub_serial: "1234567".to_owned(),
        motors: vec![
            Motor {
                address: addr("4JK"),
                name: Some("John House".to_owned()),
                version: MotorVersion {
                    address: addr("4JK"),
                    motor_type: MotorType::Dc,
                    version: "22".to_owned(),
                },
                position: None,
            },
            Motor {
                address: addr("MWX"),
                name: Some("Dining Room".to_owned()),
                version: MotorVersion {
                    address: addr("MWX"),
                    motor_type: MotorType::Dc,
                    version: "22".to_owned(),
                },
                position: None,
            },
            Motor {
                address: addr("3YC"),
                name: Some("Kitchen".to_owned()),
                version: MotorVersion {
                    address: addr("3YC"),
                    motor_type: MotorType::Dc,
                    version: "22".to_owned(),
                },
                position: None,
            },
        ],
    }
}

// -- FromStr parsing ----------------------------------------------------------

/// Verifies that a 3-character alphanumeric input parses as
/// `MotorSelector::Exact` and bypasses name resolution entirely.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn from_str_parses_three_char_address_as_exact() {
    let sel: MotorSelector = "4JK".parse().unwrap();
    assert!(matches!(sel, MotorSelector::Exact(a) if a.as_str() == "4JK"));
}

/// Verifies that a longer string (not a valid 3-char address) parses
/// as `MotorSelector::Name` for later name resolution.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn from_str_parses_longer_string_as_name() {
    let sel: MotorSelector = "kitchen".parse().unwrap();
    assert!(matches!(sel, MotorSelector::Name(ref n) if n == "kitchen"));
}

/// Verifies that a 2-char string parses as Name (too short for an
/// address).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn from_str_parses_short_string_as_name() {
    let sel: MotorSelector = "AB".parse().unwrap();
    assert!(matches!(sel, MotorSelector::Name(_)));
}

// -- resolve_against_hub_info -------------------------------------------------

/// Verifies that a unique case-insensitive substring match resolves
/// to the correct motor address.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_single_match_returns_correct_address() {
    let info = test_hub_info();
    let result = resolve_against_hub_info("kitchen", &info).unwrap();
    assert_eq!(result, addr("3YC"));
}

/// Verifies that matching is case-insensitive — "KITCHEN" matches
/// "Kitchen".
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_is_case_insensitive() {
    let info = test_hub_info();
    assert_eq!(
        resolve_against_hub_info("KITCHEN", &info).unwrap(),
        addr("3YC")
    );
    assert_eq!(
        resolve_against_hub_info("dining room", &info).unwrap(),
        addr("MWX")
    );
}

/// Verifies that a substring match works — "din" matches "Dining Room".
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_matches_substrings() {
    let info = test_hub_info();
    assert_eq!(resolve_against_hub_info("din", &info).unwrap(), addr("MWX"));
}

/// Verifies that when the name matches no motors, the error includes
/// a list of all available motors as candidates.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_no_match_returns_error_with_all_candidates() {
    let info = test_hub_info();
    let err = resolve_against_hub_info("bedroom", &info).unwrap_err();
    match err {
        CliError::MotorNameNoMatch { name, candidates } => {
            assert_eq!(name, "bedroom");
            assert_eq!(candidates.len(), 3);
            assert!(candidates.iter().any(|c| c.contains("4JK")));
            assert!(candidates.iter().any(|c| c.contains("Kitchen")));
        },
        other => panic!("expected MotorNameNoMatch, got {other:?}"),
    }
}

/// Verifies that when the name matches multiple motors, the error
/// lists the ambiguous candidates.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_ambiguous_match_returns_error_with_matching_candidates() {
    // "oo" matches both "John House" (no) and "Dining Room" (yes) and
    // "Kitchen" (no). Actually "oo" matches "Dining Room" only.
    // Let me use "o" which matches "John House" and "Dining Room".
    let info = test_hub_info();
    let err = resolve_against_hub_info("o", &info).unwrap_err();
    match err {
        CliError::MotorNameAmbiguous { name, candidates } => {
            assert_eq!(name, "o");
            assert!(candidates.len() >= 2);
        },
        other => panic!("expected MotorNameAmbiguous, got {other:?}"),
    }
}

/// Verifies that motors with no friendly name (name = None) are skipped
/// during name matching but still appear in the "no match" candidate list.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_skips_unnamed_motors_during_matching_but_lists_them_in_candidates() {
    let info = HubInfo {
        hub_name: "Test Hub".to_owned(),
        hub_serial: "1234567".to_owned(),
        motors: vec![Motor {
            address: addr("4JK"),
            name: None,
            version: MotorVersion {
                address: addr("4JK"),
                motor_type: MotorType::Dc,
                version: "22".to_owned(),
            },
            position: None,
        }],
    };
    let err = resolve_against_hub_info("kitchen", &info).unwrap_err();
    match err {
        CliError::MotorNameNoMatch { candidates, .. } => {
            assert_eq!(candidates.len(), 1);
            assert!(candidates[0].contains("4JK"));
            assert!(candidates[0].contains('?'));
        },
        other => panic!("expected MotorNameNoMatch, got {other:?}"),
    }
}

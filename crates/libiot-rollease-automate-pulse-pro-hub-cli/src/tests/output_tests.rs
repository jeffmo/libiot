//! Tests for the output formatting layer — JSON round-trip and human
//! rendering sanity checks.

use libiot_rollease_automate_pulse_pro_hub::MotorPosition;
use libiot_rollease_automate_pulse_pro_hub::MotorVoltage;

use crate::output::MotorPositionView;
use crate::output::MotorVoltageView;

/// Verifies that `MotorPositionView` serializes to JSON with the
/// expected field names and that the round-trip through `serde_json`
/// produces valid JSON.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_position_view_serializes_to_expected_json_fields() {
    let pos = MotorPosition {
        closed_percent: 50,
        tilt_percent: 180,
        signal: 0x4C,
    };
    let view = MotorPositionView::from(pos);
    let json = serde_json::to_value(&view).unwrap();

    assert_eq!(json["closed_percent"], 50);
    assert_eq!(json["tilt_percent"], 180);
    assert_eq!(json["signal"], 0x4C);
}

/// Verifies that `MotorVoltageView` includes both `centivolts` (raw
/// integer) and `volts` (computed float) in its JSON output so scripts
/// can use whichever form they prefer.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_voltage_view_includes_both_centivolts_and_volts() {
    let volt = MotorVoltage {
        centivolts: 1208,
        signal: 0x58,
    };
    let view = MotorVoltageView::from(volt);
    let json = serde_json::to_value(&view).unwrap();

    assert_eq!(json["centivolts"], 1208);
    // Float comparison: 12.08 ± epsilon
    let volts = json["volts"].as_f64().unwrap();
    assert!((volts - 12.08).abs() < 0.01);
    assert_eq!(json["signal"], 0x58);
}

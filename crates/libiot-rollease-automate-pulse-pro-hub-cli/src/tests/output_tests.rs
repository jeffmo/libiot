//! Tests for the output formatting layer — JSON round-trip, signal
//! quality labels, and human rendering sanity checks.

use libiot_rollease_automate_pulse_pro_hub::MotorPosition;
use libiot_rollease_automate_pulse_pro_hub::MotorVoltage;

use crate::output::MotorPositionView;
use crate::output::MotorVoltageView;
use crate::output::SignalView;

/// Verifies that `MotorPositionView` serializes to JSON with the
/// expected field names and that the round-trip through `serde_json`
/// produces valid JSON. The `signal` field should now be a nested
/// object with `raw` and `quality` sub-fields.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_position_view_serializes_signal_as_nested_object() {
    let pos = MotorPosition {
        closed_percent: 50,
        tilt_percent: 180,
        signal: 0x4C,
    };
    let view = MotorPositionView::from(pos);
    let json = serde_json::to_value(&view).unwrap();

    assert_eq!(json["closed_percent"], 50);
    assert_eq!(json["tilt_percent"], 180);
    assert_eq!(json["signal"]["raw"], 0x4C);
    assert_eq!(json["signal"]["quality"], "ok");
}

/// Verifies that `MotorVoltageView` includes both `centivolts` (raw
/// integer) and `volts` (computed float) in its JSON output, and that
/// `signal` is a nested object with quality label.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_voltage_view_includes_signal_quality_in_json() {
    let volt = MotorVoltage {
        centivolts: 1208,
        signal: 0x58,
    };
    let view = MotorVoltageView::from(volt);
    let json = serde_json::to_value(&view).unwrap();

    assert_eq!(json["centivolts"], 1208);
    let volts = json["volts"].as_f64().unwrap();
    assert!((volts - 12.08).abs() < 0.01);
    assert_eq!(json["signal"]["raw"], 0x58);
    assert_eq!(json["signal"]["quality"], "great");
}

/// Verifies the `signal_quality` thresholds produce the expected
/// labels at each boundary. Empirical thresholds based on real-hub
/// observations: >= 80 great, 60-79 ok, 40-59 weak, < 40 poor.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn signal_quality_labels_at_boundary_values() {
    // great: >= 80
    assert_eq!(SignalView::from_raw(88).quality, "great");
    assert_eq!(SignalView::from_raw(80).quality, "great");
    assert_eq!(SignalView::from_raw(255).quality, "great");

    // ok: 60-79
    assert_eq!(SignalView::from_raw(79).quality, "ok");
    assert_eq!(SignalView::from_raw(76).quality, "ok");
    assert_eq!(SignalView::from_raw(60).quality, "ok");

    // weak: 40-59
    assert_eq!(SignalView::from_raw(59).quality, "weak");
    assert_eq!(SignalView::from_raw(40).quality, "weak");

    // poor: 0-39
    assert_eq!(SignalView::from_raw(39).quality, "poor");
    assert_eq!(SignalView::from_raw(0).quality, "poor");
}

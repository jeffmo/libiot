//! Tests for [`crate::MotorVoltage::volts`] — the centivolt → float
//! conversion used by logging and UI.

use crate::motor_voltage::MotorVoltage;

/// Verifies that `volts()` returns the centivolt field divided by
/// 100, matching the `01208 → 12.08 V` example from §2.3 of
/// `PULSE_PRO_LOCAL_API.md`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_voltage_volts_divides_centivolts_by_100() {
    let v = MotorVoltage {
        centivolts: 1208,
        signal: 0x58,
    };
    assert!((v.volts() - 12.08).abs() < 1e-6);
}

/// Verifies `volts()` handles the zero-voltage edge case cleanly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_voltage_volts_handles_zero() {
    let v = MotorVoltage {
        centivolts: 0,
        signal: 0,
    };
    assert!(v.volts().abs() < f32::EPSILON);
}

/// Verifies `volts()` produces a sensible value at the high end of
/// the realistic range (a 12 V battery has a nominal centivolt value
/// around 1250).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn motor_voltage_volts_at_12v_battery_nominal() {
    let v = MotorVoltage {
        centivolts: 1250,
        signal: 0x58,
    };
    assert!((v.volts() - 12.5).abs() < 1e-6);
}

//! Output formatting — human-readable and JSON views.
//!
//! The CLI defines its own serializable "view" structs rather than
//! adding `serde::Serialize` to the library types. This keeps the
//! library's dependency tree minimal and lets the CLI make its own
//! decisions about JSON field names (e.g. including both `centivolts`
//! and `volts` in the voltage view even though the library only stores
//! one).

use std::fmt;
use std::str::FromStr;

use libiot_rollease_automate_pulse_pro_hub::HubInfo;
use libiot_rollease_automate_pulse_pro_hub::Motor;
use libiot_rollease_automate_pulse_pro_hub::MotorPosition;
use libiot_rollease_automate_pulse_pro_hub::MotorType;
use libiot_rollease_automate_pulse_pro_hub::MotorVersion;
use libiot_rollease_automate_pulse_pro_hub::MotorVoltage;

// ---------------------------------------------------------------------------
// OutputFormat
// ---------------------------------------------------------------------------

/// How to format output to stdout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum OutputFormat {
    /// Aligned human-readable text (default).
    #[default]
    Human,
    /// Machine-readable JSON.
    Json,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            other => Err(format!(
                "unknown output format {other:?}: expected \"human\" or \"json\""
            )),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Human => f.write_str("human"),
            Self::Json => f.write_str("json"),
        }
    }
}

// ---------------------------------------------------------------------------
// View structs (CLI-local, serde-serializable)
// ---------------------------------------------------------------------------

/// JSON view of [`HubInfo`].
#[derive(serde::Serialize)]
pub(crate) struct HubInfoView<'a> {
    pub hub_name: &'a str,
    pub hub_serial: &'a str,
    pub motors: Vec<MotorView<'a>>,
}

impl<'a> From<&'a HubInfo> for HubInfoView<'a> {
    fn from(info: &'a HubInfo) -> Self {
        Self {
            hub_name: &info.hub_name,
            hub_serial: &info.hub_serial,
            motors: info.motors.iter().map(MotorView::from).collect(),
        }
    }
}

/// JSON view of a single [`Motor`].
#[derive(serde::Serialize)]
pub(crate) struct MotorView<'a> {
    pub address: &'a str,
    pub name: Option<&'a str>,
    pub motor_type: &'static str,
    pub firmware_version: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<MotorPositionView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voltage: Option<MotorVoltageView>,
}

impl<'a> From<&'a Motor> for MotorView<'a> {
    fn from(m: &'a Motor) -> Self {
        Self {
            address: m.address.as_str(),
            name: m.name.as_deref(),
            motor_type: motor_type_str(m.version.motor_type),
            firmware_version: &m.version.version,
            position: m.position.map(MotorPositionView::from),
            voltage: m.voltage.map(MotorVoltageView::from),
        }
    }
}

/// JSON view of a [`MotorPosition`].
#[derive(serde::Serialize)]
pub(crate) struct MotorPositionView {
    pub closed_percent: u8,
    pub tilt_percent: u16,
    pub signal: SignalView,
}

impl From<MotorPosition> for MotorPositionView {
    fn from(p: MotorPosition) -> Self {
        Self {
            closed_percent: p.closed_percent,
            tilt_percent: p.tilt_percent,
            signal: SignalView::from_raw(p.signal),
        }
    }
}

/// JSON view of the undocumented RF signal-strength byte.
///
/// The raw byte is an unsigned 0–255 value representing 433 MHz ARC
/// radio link quality between the hub and the motor. The exact units
/// are not documented by Rollease. Empirically observed: 88 (0x58) on
/// a close/strong motor, 76 (0x4C) on a more distant one, 64 (0x40)
/// described as marginal. Higher = better.
#[derive(serde::Serialize)]
pub(crate) struct SignalView {
    /// Raw byte value as reported by the hub (0–255).
    pub raw: u8,
    /// Qualitative assessment based on empirical thresholds.
    pub quality: &'static str,
}

impl SignalView {
    /// Build a [`SignalView`] from the raw signal byte.
    pub(crate) fn from_raw(raw: u8) -> Self {
        Self {
            raw,
            quality: signal_quality(raw),
        }
    }
}

/// JSON view of a [`MotorVersion`].
#[derive(serde::Serialize)]
pub(crate) struct MotorVersionView<'a> {
    pub address: &'a str,
    pub motor_type: &'static str,
    pub firmware_version: &'a str,
}

impl<'a> From<&'a MotorVersion> for MotorVersionView<'a> {
    fn from(v: &'a MotorVersion) -> Self {
        Self {
            address: v.address.as_str(),
            motor_type: motor_type_str(v.motor_type),
            firmware_version: &v.version,
        }
    }
}

/// JSON view of a [`MotorVoltage`].
#[derive(serde::Serialize)]
pub(crate) struct MotorVoltageView {
    pub centivolts: u32,
    pub volts: f32,
    pub signal: SignalView,
}

impl From<MotorVoltage> for MotorVoltageView {
    fn from(v: MotorVoltage) -> Self {
        Self {
            centivolts: v.centivolts,
            volts: v.volts(),
            signal: SignalView::from_raw(v.signal),
        }
    }
}

// ---------------------------------------------------------------------------
// Render helpers
// ---------------------------------------------------------------------------

/// Render a [`HubInfo`] to stdout.
pub(crate) fn render_hub_info(info: &HubInfo, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let view = HubInfoView::from(info);
            println!(
                "{}",
                serde_json::to_string_pretty(&view).expect("HubInfoView is serializable")
            );
        },
        OutputFormat::Human => {
            println!("Hub:    {}", info.hub_name);
            println!("Serial: {}", info.hub_serial);
            println!();

            if info.motors.is_empty() {
                println!("(no paired motors)");
                return;
            }

            // Compute column widths for a tidy aligned table.
            let addr_w = info
                .motors
                .iter()
                .map(|m| m.address.as_str().len())
                .max()
                .unwrap_or(4)
                .max(4);
            let name_w = info
                .motors
                .iter()
                .map(|m| m.name.as_deref().unwrap_or("?").len())
                .max()
                .unwrap_or(4)
                .max(4);
            let battery_w = 7; // "BATTERY" header width, e.g. "12.08 V"

            println!(
                "{:<addr_w$}  {:<name_w$}  {:<battery_w$}  STATE",
                "ADDR", "NAME", "BATTERY",
            );
            println!(
                "{:<addr_w$}  {:<name_w$}  {:<battery_w$}  -----",
                "----", "----", "-------",
            );

            for motor in &info.motors {
                let name = motor.name.as_deref().unwrap_or("?");
                let battery = match motor.voltage {
                    Some(v) => format!("{:.2} V", v.volts()),
                    None => "?".to_owned(),
                };
                let state = match motor.position {
                    Some(p) => format!(
                        "closed={}%  tilt={}  signal={} ({})",
                        p.closed_percent,
                        p.tilt_percent,
                        p.signal,
                        signal_quality(p.signal),
                    ),
                    None => "(no position received)".to_owned(),
                };
                println!(
                    "{:<addr_w$}  {:<name_w$}  {:<battery_w$}  {state}",
                    motor.address, name, battery,
                );
            }
        },
    }
}

/// Render a [`MotorPosition`] to stdout.
pub(crate) fn render_motor_position(pos: MotorPosition, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let view = MotorPositionView::from(pos);
            println!(
                "{}",
                serde_json::to_string_pretty(&view).expect("MotorPositionView is serializable")
            );
        },
        OutputFormat::Human => {
            println!("closed: {}%", pos.closed_percent);
            println!("tilt:   {}%", pos.tilt_percent);
            println!("signal: {} ({})", pos.signal, signal_quality(pos.signal));
        },
    }
}

/// Render a [`MotorVersion`] to stdout.
pub(crate) fn render_motor_version(ver: &MotorVersion, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let view = MotorVersionView::from(ver);
            println!(
                "{}",
                serde_json::to_string_pretty(&view).expect("MotorVersionView is serializable")
            );
        },
        OutputFormat::Human => {
            println!("type:     {}", motor_type_str(ver.motor_type));
            println!("firmware: {}", ver.version);
        },
    }
}

/// Render a [`MotorVoltage`] to stdout.
pub(crate) fn render_motor_voltage(volt: MotorVoltage, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let view = MotorVoltageView::from(volt);
            println!(
                "{}",
                serde_json::to_string_pretty(&view).expect("MotorVoltageView is serializable")
            );
        },
        OutputFormat::Human => {
            println!(
                "voltage: {:.2} V ({} centivolts)",
                volt.volts(),
                volt.centivolts
            );
            println!("signal:  {} ({})", volt.signal, signal_quality(volt.signal));
        },
    }
}

/// Render a single string value to stdout (for `hub name`, `hub serial`,
/// `motor <m> name`).
pub(crate) fn render_string(value: &str, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).expect("string is serializable")
            );
        },
        OutputFormat::Human => {
            println!("{value}");
        },
    }
}

/// Render a simple success confirmation (for fire-and-forget commands).
pub(crate) fn render_ok(format: OutputFormat) {
    match format {
        OutputFormat::Json => println!("{{\"ok\":true}}"),
        OutputFormat::Human => {}, // silence — the command succeeded, nothing to say
    }
}

/// Qualitative assessment of the undocumented RF signal-strength byte.
///
/// The thresholds are empirical, based on real-hub observations documented
/// in `PULSE_PRO_LOCAL_API.md` §5.2: `0x58` (88) observed as strong on a
/// close motor, `0x4C` (76) as weaker on a distant one, `0x40` (64)
/// described as marginal. The exact scale is not documented by Rollease,
/// so these labels are best-effort guidance, not precise engineering data.
fn signal_quality(signal: u8) -> &'static str {
    match signal {
        80..=u8::MAX => "great",
        60..=79 => "ok",
        40..=59 => "weak",
        0..=39 => "poor",
    }
}

/// Human-readable name for a [`MotorType`].
fn motor_type_str(t: MotorType) -> &'static str {
    match t {
        MotorType::Ac => "AC",
        MotorType::Curtain => "curtain",
        MotorType::Dc => "DC",
        MotorType::DcLower => "DC (lower)",
        MotorType::DcU => "DC (U)",
        MotorType::HubGateway => "hub/gateway",
        MotorType::Light => "light",
        MotorType::Socket => "socket",
    }
}

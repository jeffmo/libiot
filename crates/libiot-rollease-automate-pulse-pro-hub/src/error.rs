//! The [`Error`] enum, the crate-local [`Result`] alias, and the
//! [`HubErrorCode`] companion enum.
//!
//! `HubErrorCode` is a small 13-variant companion to `Error` and lives in
//! this same file under the "small-companion" exception in §4 of the
//! `iot-crate-standards` skill: the type is used exclusively as a field
//! on one `Error` variant, and splitting it into its own file would be
//! more awkward than useful.

use crate::motor_address::MotorAddress;

/// Crate-local [`Result`] alias. All fallible functions in the crate
/// return `Result<T>` so callers don't have to spell the error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Every error this crate can surface.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error on the underlying TCP stream.
    #[error("I/O error talking to Pulse Pro hub: {0}")]
    Io(#[from] std::io::Error),

    /// A string was passed to [`MotorAddress::new`][crate::MotorAddress::new]
    /// or a similar constructor that did not satisfy the 3-char
    /// `[0-9A-Za-z]` constraint.
    #[error("invalid motor address {input:?}: must be 3 chars of [0-9A-Za-z]")]
    InvalidAddress {
        /// The offending input string.
        input: String,
    },

    /// A percentage argument was outside the allowed `0..=100` range.
    #[error("invalid percentage {value}: must be in 0..=100")]
    InvalidPercentage {
        /// The offending value.
        value: u16,
    },

    /// The hub returned bytes that do not match any expected frame
    /// shape. Malformed frames usually indicate a firmware quirk the
    /// parser doesn't yet handle or (rarely) a transport error that
    /// corrupted bytes mid-stream.
    #[error("hub sent malformed frame: {detail} (raw: {raw:?})")]
    Malformed {
        /// Human-readable description of what didn't parse.
        detail: String,
        /// The raw frame bytes as they appeared on the wire.
        raw: String,
    },

    /// The hub replied with a typed error response (`!<addr>E<xx>;`)
    /// for a command that targeted a specific motor.
    #[error("hub reported error {code:?} for motor {address}")]
    HubError {
        /// The motor address the error applies to.
        address: MotorAddress,
        /// The typed error code.
        code: HubErrorCode,
    },

    /// A query timed out waiting for the expected response frame.
    #[error("timed out waiting {ms}ms for response from hub")]
    Timeout {
        /// The timeout duration that elapsed, in milliseconds.
        ms: u64,
    },
}

/// Typed error codes the Pulse Pro hub can report back for a failed
/// command. Sourced from §2.8 of the in-crate `PULSE_PRO_LOCAL_API.md`.
///
/// Variants are sorted alphabetically per the workspace code-style
/// rules. The `Unknown` variant captures any code the parser doesn't
/// recognize, so the crate does not silently drop new or
/// undocumented error codes the hub might emit in future firmware.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HubErrorCode {
    /// `bz` — 433 MHz module missed a message while the Wi-Fi module
    /// was busy.
    Busy,
    /// `ec` — undefined / generic error (catch-all).
    Generic,
    /// `ph` — high-voltage alarm.
    HighVoltage,
    /// `pl` — low-voltage alarm.
    LowVoltage,
    /// `mh` — master Hall sensor abnormal.
    MasterHallSensor,
    /// `nl` — hub did not get a response from the target motor (motor
    /// offline or out of RF range).
    MotorOffline,
    /// `nc` — motor has no upper or lower limits set.
    NoLimitsSet,
    /// `np` — no such motor; the address is not in the hub's paired list.
    NoSuchMotor,
    /// `cr` — motor stalled by an obstacle during a **down** movement.
    ObstacleDown,
    /// `or` — motor stalled by an obstacle during an **up** movement.
    ObstacleUp,
    /// `sh` — slave Hall sensor abnormal.
    SlaveHallSensor,
    /// `df` — more than 30 motors paired; hub limit exceeded.
    TooManyMotors,
    /// Any 2-char error code not documented above.
    Unknown(String),
}

impl HubErrorCode {
    /// Parse a 2-character wire-format error code.
    ///
    /// Returns the typed variant for every documented code, or
    /// [`HubErrorCode::Unknown`] for anything else. This never fails —
    /// unknown codes are captured rather than dropped so forward-
    /// compatibility with future firmware is preserved.
    #[must_use]
    pub fn from_wire(code: &str) -> Self {
        match code {
            "bz" => Self::Busy,
            "cr" => Self::ObstacleDown,
            "df" => Self::TooManyMotors,
            "ec" => Self::Generic,
            "mh" => Self::MasterHallSensor,
            "nc" => Self::NoLimitsSet,
            "nl" => Self::MotorOffline,
            "np" => Self::NoSuchMotor,
            "or" => Self::ObstacleUp,
            "ph" => Self::HighVoltage,
            "pl" => Self::LowVoltage,
            "sh" => Self::SlaveHallSensor,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

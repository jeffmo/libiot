//! The [`CliError`] enum, the crate-local [`CliResult`] alias, and
//! exit-code mapping.

use libiot_rollease_automate_pulse_pro_hub::Error as LibError;

/// Every error the CLI can surface.
#[derive(Debug, thiserror::Error)]
pub(crate) enum CliError {
    /// No `--hub` flag and no `LIBIOT_PULSE_PRO_HUB` env var.
    #[error("no hub address: pass --hub <HOST[:PORT]> or set LIBIOT_PULSE_PRO_HUB")]
    NoHubAddress,

    /// The `--hub` value couldn't be parsed as a valid host\[:port\].
    #[error("invalid hub address {input:?}: {reason}")]
    InvalidHubAddress {
        /// The raw input string.
        input: String,
        /// Why it didn't parse.
        reason: String,
    },

    /// A motor friendly-name lookup matched zero paired motors.
    #[error(
        "{name:?} matched no motors. Available: {}",
        format_candidates(.candidates)
    )]
    MotorNameNoMatch {
        /// The name the user typed.
        name: String,
        /// Every paired motor formatted as `"<addr> (<name>)"`.
        candidates: Vec<String>,
    },

    /// A motor friendly-name lookup matched more than one motor.
    #[error(
        "{name:?} is ambiguous — matched {} motors: {}",
        .candidates.len(),
        format_candidates(.candidates)
    )]
    MotorNameAmbiguous {
        /// The name the user typed.
        name: String,
        /// The matching motors formatted as `"<addr> (<name>)"`.
        candidates: Vec<String>,
    },

    /// An error propagated from the library crate.
    #[error("hub: {0}")]
    Hub(#[from] LibError),
}

impl CliError {
    /// Map to a process exit code. `1` for all runtime errors; clap
    /// handles `2` for usage / argument errors on its own.
    //
    // Takes `&self` even though the current impl ignores the variant
    // because future error variants may want different exit codes.
    #[allow(clippy::unused_self)]
    pub(crate) fn exit_code(&self) -> i32 {
        1
    }

    /// A stable, grep-able string identifying the error category.
    /// Used as the `"kind"` field in `--output json` error responses.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::NoHubAddress | Self::InvalidHubAddress { .. } => "config",
            Self::MotorNameNoMatch { .. } | Self::MotorNameAmbiguous { .. } => "motor-resolution",
            Self::Hub(lib_err) => match lib_err {
                LibError::Io(_) => "io",
                LibError::Timeout { .. } => "timeout",
                LibError::HubError { .. } => "hub-error",
                LibError::Malformed { .. } => "malformed",
                LibError::InvalidAddress { .. } => "invalid-address",
                LibError::InvalidPercentage { .. } => "invalid-percentage",
            },
        }
    }
}

/// Crate-local [`Result`] alias.
pub(crate) type CliResult<T> = std::result::Result<T, CliError>;

/// Format a list of motor candidates as a comma-separated string for
/// the `#[error(...)]` `Display` impl. Each candidate is expected to
/// be pre-formatted as `"<addr> (<name>)"` by the motor-selector
/// layer.
fn format_candidates(candidates: &[String]) -> String {
    candidates.join(", ")
}

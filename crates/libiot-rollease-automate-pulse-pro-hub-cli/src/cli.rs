//! Top-level clap [`Cli`] struct and the [`Command`] subcommand enum.

use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;

use crate::motor_selector::MotorSelector;
use crate::output::OutputFormat;

/// Control a Rollease Acmeda Automate Pulse Pro shade hub from the
/// command line.
///
/// Every operation the library supports is exposed as a subcommand.
/// Motor arguments accept either a 3-char hub-assigned address (e.g.
/// `4JK`) or a friendly name (e.g. `kitchen`) which is resolved via
/// case-insensitive substring matching against the hub's paired motor
/// list.
#[derive(Parser, Debug)]
#[command(
    name = "libiot-rollease-automate-pulse-pro-hub",
    version,
    about = "Control a Rollease Acmeda Automate Pulse Pro shade hub from the command line"
)]
pub(crate) struct Cli {
    /// Hub address as HOST\[:PORT\]. Port defaults to 1487 if omitted.
    #[arg(long, env = "LIBIOT_PULSE_PRO_HUB", global = true)]
    pub hub: Option<String>,

    /// Output format: "human" (default) for aligned text, "json" for
    /// machine-readable JSON.
    #[arg(long, default_value = "human", global = true)]
    pub format: OutputFormat,

    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level subcommand dispatch.
#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    // -- motor control (flat, most common) ----------------------------
    /// Fully open a motor (lift to 0% closed).
    Open {
        /// Motor address or friendly name.
        motor: MotorSelector,
    },

    /// Fully close a motor (lift to 100% closed).
    Close {
        /// Motor address or friendly name.
        motor: MotorSelector,
    },

    /// Stop any in-flight movement on a motor.
    Stop {
        /// Motor address or friendly name.
        motor: MotorSelector,
    },

    /// Move a motor to a specific closed-lift percentage (0-100).
    ///
    /// The command is fire-and-forget — the CLI exits as soon as the
    /// command is sent. The shade may still be moving when the CLI
    /// returns. Use `motor <motor> position` to poll the current state.
    SetPosition {
        /// Motor address or friendly name.
        motor: MotorSelector,
        /// Closed-lift percentage: 0 = fully open, 100 = fully closed.
        percent: u8,
    },

    /// Set the slat tilt angle on a venetian blind (0-100).
    ///
    /// "Tilt" refers to the angle of individual slats on venetian
    /// blinds (horizontal blinds) or shutters. 0 = slats
    /// horizontal/open (maximum light), 100 = slats fully
    /// angled/closed (minimum light). Tilt only applies to slatted
    /// shades — roller shades, cellular shades, and drapes silently
    /// ignore this command.
    SetTilt {
        /// Motor address or friendly name.
        motor: MotorSelector,
        /// Tilt percentage: 0 = slats open, 100 = slats closed.
        percent: u8,
    },

    /// Nudge a motor one step open.
    JogOpen {
        /// Motor address or friendly name.
        motor: MotorSelector,
    },

    /// Nudge a motor one step closed.
    JogClose {
        /// Motor address or friendly name.
        motor: MotorSelector,
    },

    // -- broadcast (flat) ---------------------------------------------
    /// Open every paired motor.
    OpenAll,

    /// Close every paired motor.
    CloseAll,

    /// Stop every paired motor.
    StopAll,

    // -- hub info (flat, high-use) --------------------------------------
    /// Full hub info: name, serial, and every paired motor with
    /// position, friendly name, and battery voltage.
    ///
    /// Each motor's signal quality label is an empirical estimate —
    /// see `motor <MOTOR> position --help` for threshold details.
    Info,

    // -- hub queries (nested under `hub`) -----------------------------
    /// Query individual hub-level attributes.
    Hub {
        /// Which hub query to run.
        #[command(subcommand)]
        query: HubQuery,
    },

    // -- per-motor queries (nested under `motor`) ---------------------
    /// Query a specific motor. Without a sub-query, shows all info
    /// (name, version, position, voltage) for the motor.
    Motor {
        /// Motor address or friendly name.
        motor: MotorSelector,
        /// Which motor query to run (omit for all info).
        #[command(subcommand)]
        query: Option<MotorQuery>,
    },

    // -- shell completions (hidden) -----------------------------------
    /// Generate shell completions.
    #[command(hide = true)]
    Completions {
        /// Target shell (omit for setup instructions).
        shell: Option<clap_complete::Shell>,
        /// Print only the source snippet (for piping to shell config).
        #[arg(long)]
        print_config: bool,
    },

    /// Write a man page to a file.
    #[command(hide = true)]
    Man {
        /// Output path for the man page.
        path: PathBuf,
    },

    // -- dangerous-ops (feature-gated) --------------------------------
    /// Destructive hub operations (pair, unpair, delete, factory
    /// reset, limit configuration). Only available when built with
    /// `--features dangerous-ops`.
    #[cfg(feature = "dangerous-ops")]
    Dangerous {
        /// Which dangerous operation to run.
        #[command(subcommand)]
        op: DangerousOp,
    },
}

/// Hub-level query subcommands.
#[derive(Subcommand, Debug)]
pub(crate) enum HubQuery {
    /// Hub friendly name only.
    Name,
    /// Hub serial number only.
    Serial,
}

/// Per-motor query subcommands.
#[derive(Subcommand, Debug)]
pub(crate) enum MotorQuery {
    /// Motor's friendly name.
    Name,

    /// Current position (closed%, tilt%, signal strength).
    ///
    /// The signal value is the undocumented RF link-quality byte on
    /// the hub's 433 MHz ARC radio link to the motor. Rollease does
    /// not publish its units or scale. The qualitative labels shown
    /// alongside the value are based on empirical observations from
    /// real hardware and should be treated as rough guidance, not
    /// precise engineering data.
    ///
    /// ```text
    ///   >= 80  "great"  (observed on motors physically close to the hub)
    ///   60-79  "ok"     (observed on more distant motors)
    ///   40-59  "weak"   (described as marginal in community references)
    ///   < 40   "poor"
    /// ```
    #[command(long_about = "\
Current position (closed%, tilt%, signal strength).

The signal value is the undocumented RF link-quality byte on the hub's \
433 MHz ARC radio link to the motor. Rollease does not publish its units \
or scale. The qualitative labels shown alongside the value are based on \
empirical observations from real hardware and should be treated as rough \
guidance, not precise engineering data:

  >= 80  \"great\"  (observed on motors physically close to the hub)
  60-79  \"ok\"     (observed on more distant motors)
  40-59  \"weak\"   (described as marginal in community references)
  < 40   \"poor\"")]
    Position,

    /// Motor type and firmware version.
    Version,

    /// Battery voltage and signal strength.
    ///
    /// Signal thresholds are the same as for the `position` subcommand.
    #[command(long_about = "\
Battery voltage and signal strength.

Signal thresholds are the same as for the `position` subcommand — see \
`motor <MOTOR> position --help` for details.")]
    Voltage,
}

/// Destructive hub operations — feature-gated behind `dangerous-ops`.
#[cfg(feature = "dangerous-ops")]
#[derive(Subcommand, Debug)]
pub(crate) enum DangerousOp {
    /// Pair a new motor. The hub auto-assigns a 3-char address.
    Pair,
    /// Unpair a motor (requires the motor to acknowledge).
    Unpair {
        /// Motor address or friendly name.
        motor: MotorSelector,
    },
    /// Delete a motor from the hub's paired list (no motor ack required).
    Delete {
        /// Motor address or friendly name.
        motor: MotorSelector,
    },
    /// Factory-reset the hub — wipes all pairing configuration.
    FactoryReset,
    /// Set a motor's upper limit at its current position.
    SetUpperLimit {
        /// Motor address or friendly name.
        motor: MotorSelector,
    },
    /// Set a motor's lower limit at its current position.
    SetLowerLimit {
        /// Motor address or friendly name.
        motor: MotorSelector,
    },
    /// Delete all of a motor's configured limits.
    DeleteLimits {
        /// Motor address or friendly name.
        motor: MotorSelector,
    },
}

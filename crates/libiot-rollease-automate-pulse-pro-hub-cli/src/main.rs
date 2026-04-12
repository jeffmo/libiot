#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Command-line interface for the Rollease Acmeda Automate Pulse Pro
//! shade hub. Wraps every operation in the
//! [`libiot-rollease-automate-pulse-pro-hub`] library crate as an
//! ergonomic CLI subcommand.
//!
//! # Examples
//!
//! ```bash
//! # Close the kitchen shade by friendly name
//! libiot-rollease-automate-pulse-pro-hub --hub 192.168.5.234 close kitchen
//!
//! # Query full hub info (name, serial, all motors + positions)
//! libiot-rollease-automate-pulse-pro-hub --hub 192.168.5.234 hub info
//!
//! # Query a specific motor's position by its 3-char address
//! libiot-rollease-automate-pulse-pro-hub --hub 192.168.5.234 motor 3YC position
//! ```
//!
//! Set `LIBIOT_PULSE_PRO_HUB=192.168.5.234` to avoid repeating `--hub`
//! on every invocation.

mod cli;
mod commands;
mod error;
mod hub_connection;
mod motor_selector;
mod output;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cli::Cli;
use crate::error::CliError;

/// Entry point. Parses CLI arguments via clap, runs the selected
/// subcommand inside a single-threaded tokio runtime, and maps errors
/// to exit codes.
fn main() {
    let cli = Cli::parse();
    let output_format = cli.format;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let result = rt.block_on(commands::run(cli));

    if let Err(err) = result {
        report_error(&err, output_format);
        std::process::exit(err.exit_code());
    }
}

/// Print an error to stderr. In JSON mode, emit a structured JSON
/// object with `error` (human message) and `kind` (stable category
/// string) so scripts can parse it.
fn report_error(err: &CliError, format: crate::output::OutputFormat) {
    match format {
        crate::output::OutputFormat::Json => {
            let json = serde_json::json!({
                "error": err.to_string(),
                "kind": err.kind(),
            });
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&json).expect("JSON error is serializable")
            );
        },
        crate::output::OutputFormat::Human => {
            eprintln!("error: {err}");
        },
    }
}

//! Built-in command dispatch.
//!
//! Each built-in subcommand is handled by a dedicated function (or
//! sub-module) invoked from [`run`]. Delegation to discovered
//! `libiot-*-cli` binaries is handled in the top-level dispatcher
//! (`main.rs`), not here.

/// Execute the built-in command described by `cli`.
#[allow(clippy::needless_pass_by_value)] // will consume fields once handlers land
pub(crate) fn run(cli: crate::cli::Cli) -> crate::error::CliResult<()> {
    let _ = cli;
    todo!("command dispatch not yet implemented")
}

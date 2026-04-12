//! Built-in command dispatch.
//!
//! Each built-in subcommand is handled by a dedicated function (or
//! sub-module) invoked from [`run`]. Delegation to discovered
//! `libiot-*-cli` binaries is handled in the top-level dispatcher
//! (`main.rs`), not here.

mod completions;
mod get;
mod install;
mod list;
mod set;
mod uninstall;
mod unset;

#[cfg(test)]
mod tests;

use crate::cli::Cli;
use crate::cli::Command;
use crate::error::CliResult;
use crate::output::OutputContext;

/// Execute the built-in command described by `cli`.
pub(crate) fn run(cli: Cli) -> CliResult<()> {
    let ctx = OutputContext {
        format: cli.format,
        quiet: cli.quiet,
    };
    match cli.command {
        Command::Set { target } => set::run_set(target, ctx),
        Command::Unset { target } => unset::run_unset(target, ctx),
        Command::Get { target } => get::run_get(target, ctx),
        Command::List { target } => list::run_list(target, ctx),
        Command::Install(ref args) => install::run_install(args, ctx),
        Command::Uninstall(ref args) => uninstall::run_uninstall(args, ctx),
        Command::Completions { shell } => {
            completions::run_completions(shell);
            Ok(())
        },
        Command::ConfigPath => {
            let path = crate::settings::settings_path()?;
            crate::output::render_config_path(&path, ctx);
            Ok(())
        },
    }
}

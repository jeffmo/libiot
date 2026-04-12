#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Unified CLI dispatcher for the libiot ecosystem.
//!
//! `libiot` discovers all installed `libiot-*` CLI binaries on `$PATH`
//! and exposes them as subcommands, with alias management, per-command
//! environment variable injection, and `cargo install`/`uninstall`
//! wrappers.

mod cli;
mod commands;
mod discovery;
mod error;
mod output;
mod settings;

#[cfg(test)]
mod tests;

use std::ffi::OsString;

use clap::Parser;

use crate::cli::is_builtin;
use crate::cli::Cli;
use crate::output::report_error;
use crate::output::OutputContext;

/// Entry point.
///
/// Implements pre-parse dispatch with two operating modes:
///
/// 1. **Built-in mode** — `argv[1]` starts with `-` or is a known
///    built-in command name. The full argv is parsed with clap and
///    dispatched to the matching command handler.
/// 2. **Delegation mode** — `argv[1]` is anything else. `argv[1..]`
///    is passed verbatim to the delegation module for exec handoff.
fn main() {
    let raw_args: Vec<OsString> = std::env::args_os().collect();

    // No args at all — let clap show help and exit.
    if raw_args.len() < 2 {
        // `arg_required_else_help = true` on the Cli struct means clap
        // will print usage and exit(2) automatically.
        Cli::parse();
        return;
    }

    let first_arg = raw_args[1].to_string_lossy();

    if first_arg.starts_with('-') || is_builtin(&first_arg) {
        // Built-in mode: full clap parse.
        match Cli::try_parse() {
            Ok(cli) => {
                let ctx = OutputContext {
                    format: cli.format,
                    quiet: cli.quiet,
                };
                if let Err(err) = commands::run(cli) {
                    report_error(&err, ctx.format);
                    std::process::exit(err.exit_code());
                }
            },
            Err(clap_err) => {
                // The user may have placed top-level flags before a
                // delegation target. Try to surface a helpful hint.
                maybe_hint_delegation(&raw_args, &clap_err);
            },
        }
    } else {
        // Delegation mode: pass argv[1..] verbatim to the delegate.
        run_delegation(&raw_args[1..]);
    }
}

/// When clap rejects the invocation, scan the remaining args for a
/// non-flag token that looks like it could be a delegation target. If
/// found, emit a hint suggesting the user move flags after the command
/// name. Otherwise, fall through to clap's default error output.
fn maybe_hint_delegation(raw_args: &[OsString], clap_err: &clap::Error) {
    // Find the first flag (starts with '-') and the first non-flag
    // token after argv[0] that is not a built-in name. That non-flag
    // token is the likely delegation target.
    let mut first_flag: Option<String> = None;
    let mut likely_command: Option<String> = None;

    for arg in &raw_args[1..] {
        let s = arg.to_string_lossy();
        if s.starts_with('-') {
            if first_flag.is_none() {
                first_flag = Some(s.into_owned());
            }
        } else if !is_builtin(&s) && likely_command.is_none() {
            likely_command = Some(s.into_owned());
        }
    }

    if let (Some(flag), Some(cmd)) = (first_flag, likely_command) {
        eprintln!("error: unknown flag {flag:?} before command {cmd:?}");
        eprintln!();
        eprintln!(
            "  Top-level flags like {flag} are not allowed when \
             delegating to a"
        );
        eprintln!("  sub-CLI. Place all flags after the command name:");
        eprintln!();
        eprintln!("    libiot {cmd} {flag} ...");
        std::process::exit(2);
    }

    // No likely delegation target found — let clap print its own
    // error and exit.
    clap_err.exit();
}

/// Delegate execution to a discovered `libiot-*-cli` binary.
///
/// This is a stub that will be replaced by the full delegation module.
fn run_delegation(args: &[OsString]) -> ! {
    // Delegation module not yet implemented.
    let name = args[0].to_string_lossy();
    eprintln!("error: delegation not yet implemented for {name:?}");
    std::process::exit(1)
}

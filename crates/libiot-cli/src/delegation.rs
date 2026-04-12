//! Exec handoff for delegation mode.
//!
//! When the user invokes `libiot <name> [args...]` and `<name>` is not
//! a built-in command, the CLI resolves the name to a `libiot-*` binary
//! (possibly through an alias), injects per-command environment
//! variables from settings, and replaces the current process via
//! `exec()`.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

use crate::discovery::find_cli_with_path;
use crate::error::CliError;
use crate::error::CliResult;
use crate::settings::Settings;
use crate::settings::load_settings;
use crate::settings::resolve_env_vars;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Resolve `name` to a binary, inject env vars, and replace this
/// process (exec handoff). Does not return on success.
pub(crate) fn delegate(args: &[OsString]) -> CliResult<()> {
    let name = args[0].to_string_lossy().into_owned();
    let remaining = &args[1..];

    let settings = load_settings()?;

    let path_str = std::env::var("PATH").unwrap_or_default();
    let (binary_path, env_vars) = resolve_delegation_with(&name, &settings, &path_str)?;

    let mut cmd = Command::new(&binary_path);
    cmd.args(remaining);
    for (k, v) in &env_vars {
        cmd.env(k, v);
    }

    exec_handoff(&mut cmd)
}

// ---------------------------------------------------------------------------
// Testable resolution logic
// ---------------------------------------------------------------------------

/// Resolve a delegation target to a binary path and environment
/// variables, without performing the exec.
///
/// Returns `(binary_path, env_vars)` on success. This is the pure-logic
/// core of [`delegate`], extracted so tests can exercise resolution
/// without replacing the process.
pub(crate) fn resolve_delegation_with(
    name: &str,
    settings: &Settings,
    path_str: &str,
) -> CliResult<(PathBuf, BTreeMap<String, String>)> {
    // Resolve through aliases: if `name` is an alias, the underlying
    // command is what we look up on PATH.
    let resolved_name = settings.aliases.get(name).map_or(name, String::as_str);

    let binary_path = find_cli_with_path(resolved_name, path_str).ok_or_else(|| {
        CliError::DelegationTargetNotFound {
            name: name.to_owned(),
        }
    })?;

    let env_vars = resolve_env_vars(settings, name);

    Ok((binary_path, env_vars))
}

// ---------------------------------------------------------------------------
// Platform-specific exec handoff
// ---------------------------------------------------------------------------

/// Replace the current process with `cmd` via the Unix `exec()` syscall.
///
/// On success this function never returns — the current process image
/// is replaced entirely. It only returns `Err` if the `exec` syscall
/// itself fails.
#[cfg(unix)]
fn exec_handoff(cmd: &mut Command) -> CliResult<()> {
    use std::os::unix::process::CommandExt;
    // exec() replaces the process. It only returns if it fails.
    let err = cmd.exec();
    Err(CliError::ExecFailed {
        name: format!("{cmd:?}"),
        source: err,
    })
}

/// Non-Unix stub — delegation requires Unix `exec()`.
#[cfg(not(unix))]
fn exec_handoff(_cmd: &mut Command) -> CliResult<()> {
    Err(CliError::ExecFailed {
        name: String::new(),
        source: std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "delegation requires Unix exec() — this platform is not supported",
        ),
    })
}

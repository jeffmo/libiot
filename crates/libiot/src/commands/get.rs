//! Handler for the `get` subcommand — query aliases and env vars.

use crate::cli::GetTarget;
use crate::error::CliError;
use crate::error::CliResult;
use crate::output::OutputContext;
use crate::settings::load_settings;

/// Execute a `get` command for the given target.
///
/// Loads persistent settings and renders the requested alias or
/// environment variable value. Returns an error if the requested
/// item does not exist.
pub(crate) fn run_get(target: GetTarget, ctx: OutputContext) -> CliResult<()> {
    match target {
        GetTarget::Alias { alias_name } => get_alias(&alias_name, ctx),
        GetTarget::EnvVar {
            cmd_or_alias,
            var_name,
        } => get_env_var(&cmd_or_alias, &var_name, ctx),
    }
}

/// Look up a single alias and render its target.
fn get_alias(alias_name: &str, ctx: OutputContext) -> CliResult<()> {
    let settings = load_settings()?;
    let target = settings
        .aliases
        .get(alias_name)
        .ok_or_else(|| CliError::AliasNotFound {
            alias: alias_name.to_owned(),
        })?;
    crate::output::render_alias(alias_name, target, ctx);
    Ok(())
}

/// Look up a single environment variable for a command or alias and
/// render its value.
fn get_env_var(cmd_or_alias: &str, var_name: &str, ctx: OutputContext) -> CliResult<()> {
    let settings = load_settings()?;
    let value = settings
        .env_vars
        .get(cmd_or_alias)
        .and_then(|inner| inner.get(var_name))
        .ok_or_else(|| CliError::EnvVarNotFound {
            cmd_or_alias: cmd_or_alias.to_owned(),
            name: var_name.to_owned(),
        })?;
    crate::output::render_env_var_value(var_name, value, ctx);
    Ok(())
}

// ---------------------------------------------------------------------------
// Path-parameterized variants for testing
// ---------------------------------------------------------------------------

/// Same as [`run_get`] but loads settings from a specific path.
///
/// Exposed for tests so they can avoid mutating process-global env vars.
#[cfg(test)]
pub(super) fn run_get_from(
    target: GetTarget,
    ctx: OutputContext,
    settings_path: &std::path::Path,
) -> CliResult<()> {
    match target {
        GetTarget::Alias { alias_name } => get_alias_from(&alias_name, ctx, settings_path),
        GetTarget::EnvVar {
            cmd_or_alias,
            var_name,
        } => get_env_var_from(&cmd_or_alias, &var_name, ctx, settings_path),
    }
}

#[cfg(test)]
fn get_alias_from(
    alias_name: &str,
    ctx: OutputContext,
    settings_path: &std::path::Path,
) -> CliResult<()> {
    let settings = crate::settings::load_settings_from(settings_path)?;
    let target = settings
        .aliases
        .get(alias_name)
        .ok_or_else(|| CliError::AliasNotFound {
            alias: alias_name.to_owned(),
        })?;
    crate::output::render_alias(alias_name, target, ctx);
    Ok(())
}

#[cfg(test)]
fn get_env_var_from(
    cmd_or_alias: &str,
    var_name: &str,
    ctx: OutputContext,
    settings_path: &std::path::Path,
) -> CliResult<()> {
    let settings = crate::settings::load_settings_from(settings_path)?;
    let value = settings
        .env_vars
        .get(cmd_or_alias)
        .and_then(|inner| inner.get(var_name))
        .ok_or_else(|| CliError::EnvVarNotFound {
            cmd_or_alias: cmd_or_alias.to_owned(),
            name: var_name.to_owned(),
        })?;
    crate::output::render_env_var_value(var_name, value, ctx);
    Ok(())
}

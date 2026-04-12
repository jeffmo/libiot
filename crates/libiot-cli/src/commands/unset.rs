//! Handler for the `unset` subcommand — remove aliases and env vars.

use crate::cli::UnsetTarget;
use crate::error::CliError;
use crate::error::CliResult;
use crate::output::OutputContext;
use crate::output::render_ok_message;
use crate::settings::load_settings;
use crate::settings::save_settings;

/// Execute an `unset` command for the given target.
///
/// Loads settings, removes the requested item, writes settings back,
/// and renders a success message. Returns an error if the item does
/// not exist.
pub(crate) fn run_unset(target: UnsetTarget, ctx: OutputContext) -> CliResult<()> {
    match target {
        UnsetTarget::Alias { alias_name } => unset_alias(&alias_name, ctx),
        UnsetTarget::EnvVar {
            cmd_or_alias,
            var_name,
        } => unset_env_var(&cmd_or_alias, &var_name, ctx),
    }
}

/// Remove an alias mapping.
fn unset_alias(alias_name: &str, ctx: OutputContext) -> CliResult<()> {
    let mut settings = load_settings()?;

    if settings.aliases.remove(alias_name).is_none() {
        return Err(CliError::AliasNotFound {
            alias: alias_name.to_owned(),
        });
    }

    save_settings(&settings)?;

    render_ok_message(&format!("Alias {alias_name:?} removed."), ctx);
    Ok(())
}

/// Remove a single environment variable for a command or alias.
fn unset_env_var(cmd_or_alias: &str, var_name: &str, ctx: OutputContext) -> CliResult<()> {
    let mut settings = load_settings()?;

    let inner =
        settings
            .env_vars
            .get_mut(cmd_or_alias)
            .ok_or_else(|| CliError::EnvVarNotFound {
                cmd_or_alias: cmd_or_alias.to_owned(),
                name: var_name.to_owned(),
            })?;

    if inner.remove(var_name).is_none() {
        return Err(CliError::EnvVarNotFound {
            cmd_or_alias: cmd_or_alias.to_owned(),
            name: var_name.to_owned(),
        });
    }

    // Clean up the outer key when the inner map becomes empty.
    if inner.is_empty() {
        settings.env_vars.remove(cmd_or_alias);
    }

    save_settings(&settings)?;

    render_ok_message(
        &format!("Environment variable {var_name:?} removed for {cmd_or_alias:?}."),
        ctx,
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Path-parameterized variants for testing
// ---------------------------------------------------------------------------

/// Same as [`run_unset`] but loads/saves settings from a specific path.
///
/// Exposed for tests so they can avoid mutating process-global env vars.
#[cfg(test)]
pub(super) fn run_unset_from(
    target: UnsetTarget,
    ctx: OutputContext,
    settings_path: &std::path::Path,
) -> CliResult<()> {
    match target {
        UnsetTarget::Alias { alias_name } => unset_alias_from(&alias_name, ctx, settings_path),
        UnsetTarget::EnvVar {
            cmd_or_alias,
            var_name,
        } => unset_env_var_from(&cmd_or_alias, &var_name, ctx, settings_path),
    }
}

#[cfg(test)]
fn unset_alias_from(
    alias_name: &str,
    ctx: OutputContext,
    settings_path: &std::path::Path,
) -> CliResult<()> {
    let mut settings = crate::settings::load_settings_from(settings_path)?;

    if settings.aliases.remove(alias_name).is_none() {
        return Err(CliError::AliasNotFound {
            alias: alias_name.to_owned(),
        });
    }

    crate::settings::save_settings_to(&settings, settings_path)?;

    render_ok_message(&format!("Alias {alias_name:?} removed."), ctx);
    Ok(())
}

#[cfg(test)]
fn unset_env_var_from(
    cmd_or_alias: &str,
    var_name: &str,
    ctx: OutputContext,
    settings_path: &std::path::Path,
) -> CliResult<()> {
    let mut settings = crate::settings::load_settings_from(settings_path)?;

    let inner =
        settings
            .env_vars
            .get_mut(cmd_or_alias)
            .ok_or_else(|| CliError::EnvVarNotFound {
                cmd_or_alias: cmd_or_alias.to_owned(),
                name: var_name.to_owned(),
            })?;

    if inner.remove(var_name).is_none() {
        return Err(CliError::EnvVarNotFound {
            cmd_or_alias: cmd_or_alias.to_owned(),
            name: var_name.to_owned(),
        });
    }

    if inner.is_empty() {
        settings.env_vars.remove(cmd_or_alias);
    }

    crate::settings::save_settings_to(&settings, settings_path)?;

    render_ok_message(
        &format!("Environment variable {var_name:?} removed for {cmd_or_alias:?}."),
        ctx,
    );
    Ok(())
}

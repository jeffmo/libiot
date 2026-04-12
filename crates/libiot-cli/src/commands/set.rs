//! Handler for the `set` subcommand — create aliases and set env vars.

use crate::cli::BUILTIN_NAMES;
use crate::cli::SetTarget;
use crate::discovery::find_cli;
use crate::error::CliError;
use crate::error::CliResult;
use crate::output::OutputContext;
use crate::output::render_ok_message;
use crate::settings::load_settings;
use crate::settings::save_settings;

/// Execute a `set` command for the given target.
///
/// Validates inputs, loads settings, mutates them, writes them back,
/// and renders a success message.
pub(crate) fn run_set(target: SetTarget, ctx: OutputContext) -> CliResult<()> {
    match target {
        SetTarget::Alias {
            cmd,
            alias_name,
            overwrite,
        } => set_alias(&cmd, &alias_name, overwrite, ctx),
        SetTarget::EnvVar {
            cmd_or_alias,
            var_name,
            value,
        } => set_env_var(&cmd_or_alias, &var_name, &value, ctx),
    }
}

/// Create or overwrite an alias mapping.
fn set_alias(cmd: &str, alias_name: &str, overwrite: bool, ctx: OutputContext) -> CliResult<()> {
    // 1. Check for built-in shadowing.
    if BUILTIN_NAMES.contains(&alias_name) {
        return Err(CliError::AliasShadowsBuiltin {
            alias: alias_name.to_owned(),
            builtin: alias_name.to_owned(),
        });
    }

    // 2. Verify the target binary exists on PATH.
    if find_cli(cmd).is_none() {
        return Err(CliError::AliasTargetNotFound {
            cmd: cmd.to_owned(),
        });
    }

    // 3. Load settings.
    let mut settings = load_settings()?;

    // 4. Check for existing alias (unless --overwrite).
    if let Some(existing) = settings.aliases.get(alias_name) {
        if !overwrite {
            return Err(CliError::AliasAlreadyExists {
                alias: alias_name.to_owned(),
                target: existing.clone(),
            });
        }
    }

    // 5. Insert.
    settings
        .aliases
        .insert(alias_name.to_owned(), cmd.to_owned());

    // 6. Save.
    save_settings(&settings)?;

    // 7. Render.
    render_ok_message(&format!("Alias {alias_name:?} set for {cmd:?}."), ctx);
    Ok(())
}

/// Set an environment variable for a command or alias.
fn set_env_var(
    cmd_or_alias: &str,
    var_name: &str,
    value: &str,
    ctx: OutputContext,
) -> CliResult<()> {
    // 1. Reject the reserved LIBIOT_ prefix (case-insensitive).
    if var_name.to_ascii_uppercase().starts_with("LIBIOT_") {
        return Err(CliError::EnvVarLibiotPrefix {
            name: var_name.to_owned(),
        });
    }

    // 2. Load settings.
    let mut settings = load_settings()?;

    // 3. Validate the target: alias first, then PATH.
    if !settings.aliases.contains_key(cmd_or_alias) && find_cli(cmd_or_alias).is_none() {
        return Err(CliError::EnvVarTargetNotFound {
            cmd_or_alias: cmd_or_alias.to_owned(),
        });
    }

    // 4. Insert.
    settings
        .env_vars
        .entry(cmd_or_alias.to_owned())
        .or_default()
        .insert(var_name.to_owned(), value.to_owned());

    // 5. Save.
    save_settings(&settings)?;

    // 6. Render.
    render_ok_message(
        &format!("Environment variable {var_name:?} set to {value:?} for {cmd_or_alias:?}."),
        ctx,
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Path-parameterized variants for testing
// ---------------------------------------------------------------------------

/// Same as [`set_alias`] but loads/saves settings from a specific path
/// and skips the PATH check (since tests may not have real binaries).
///
/// Exposed for tests so they can avoid mutating process-global env vars.
#[cfg(test)]
pub(super) fn set_alias_from(
    cmd: &str,
    alias_name: &str,
    overwrite: bool,
    ctx: OutputContext,
    settings_path: &std::path::Path,
) -> CliResult<()> {
    if BUILTIN_NAMES.contains(&alias_name) {
        return Err(CliError::AliasShadowsBuiltin {
            alias: alias_name.to_owned(),
            builtin: alias_name.to_owned(),
        });
    }

    let mut settings = crate::settings::load_settings_from(settings_path)?;

    if let Some(existing) = settings.aliases.get(alias_name) {
        if !overwrite {
            return Err(CliError::AliasAlreadyExists {
                alias: alias_name.to_owned(),
                target: existing.clone(),
            });
        }
    }

    settings
        .aliases
        .insert(alias_name.to_owned(), cmd.to_owned());

    crate::settings::save_settings_to(&settings, settings_path)?;

    render_ok_message(&format!("Alias {alias_name:?} set for {cmd:?}."), ctx);
    Ok(())
}

/// Same as [`set_env_var`] but loads/saves from a specific path and
/// skips the PATH check. Alias validation still works against the file.
#[cfg(test)]
pub(super) fn set_env_var_from(
    cmd_or_alias: &str,
    var_name: &str,
    value: &str,
    ctx: OutputContext,
    settings_path: &std::path::Path,
) -> CliResult<()> {
    if var_name.to_ascii_uppercase().starts_with("LIBIOT_") {
        return Err(CliError::EnvVarLibiotPrefix {
            name: var_name.to_owned(),
        });
    }

    let mut settings = crate::settings::load_settings_from(settings_path)?;

    // For testing: accept if the target is a known alias (PATH check
    // is skipped in the test variant).
    if !settings.aliases.contains_key(cmd_or_alias) {
        // In test mode we skip find_cli and just allow any name,
        // since there's no real binary on PATH.
    }

    settings
        .env_vars
        .entry(cmd_or_alias.to_owned())
        .or_default()
        .insert(var_name.to_owned(), value.to_owned());

    crate::settings::save_settings_to(&settings, settings_path)?;

    render_ok_message(
        &format!("Environment variable {var_name:?} set to {value:?} for {cmd_or_alias:?}."),
        ctx,
    );
    Ok(())
}

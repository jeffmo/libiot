//! Handler for the `list` subcommand — enumerate CLIs, aliases, and
//! env vars.

use crate::cli::ListTarget;
use crate::discovery::discover_clis;
use crate::error::CliResult;
use crate::output::AliasView;
use crate::output::CliView;
use crate::output::EnvVarView;
use crate::output::OutputContext;
use crate::settings::Settings;
use crate::settings::load_settings;
use crate::settings::resolve_env_vars;

/// Execute a `list` command for the given target.
///
/// When no target is supplied, lists all discovered CLIs and aliases.
/// Otherwise narrows to aliases or environment variables.
pub(crate) fn run_list(target: Option<ListTarget>, ctx: OutputContext) -> CliResult<()> {
    match target {
        None => list_all(ctx),
        Some(ListTarget::Aliases) => list_aliases(ctx),
        Some(ListTarget::EnvVars { cmd_or_alias }) => list_env_vars(cmd_or_alias, ctx),
    }
}

/// List all discovered CLIs and configured aliases.
///
/// Also triggers a background regeneration of completion files (if
/// any exist on disk) so that tab-completion stays in sync with
/// the current set of installed CLIs and aliases.
fn list_all(ctx: OutputContext) -> CliResult<()> {
    let discovered = discover_clis();
    let settings = load_settings()?;

    let cli_views: Vec<CliView<'_>> = discovered
        .iter()
        .map(|c| CliView {
            name: &c.name,
            path: &c.path,
        })
        .collect();

    let alias_views: Vec<AliasView<'_>> = settings
        .aliases
        .iter()
        .map(|(alias, target)| AliasView {
            alias: alias.as_str(),
            target: target.as_str(),
        })
        .collect();

    crate::output::render_list_all(&cli_views, &alias_views, ctx);

    // Opportunistically regenerate completions in the background.
    crate::commands::completions::regenerate_existing_completions(/* verbose = */ false);

    Ok(())
}

/// List all configured aliases.
fn list_aliases(ctx: OutputContext) -> CliResult<()> {
    let settings = load_settings()?;
    let views = alias_views_from_settings(&settings);
    crate::output::render_list_aliases(&views, ctx);
    Ok(())
}

/// List environment variables, either for a specific command/alias or
/// for all configured groups.
#[allow(clippy::needless_pass_by_value)] // clap gives us an owned Option<String>
fn list_env_vars(cmd_or_alias: Option<String>, ctx: OutputContext) -> CliResult<()> {
    let settings = load_settings()?;
    list_env_vars_inner(cmd_or_alias.as_deref(), &settings, ctx);
    Ok(())
}

/// Shared inner logic for listing env vars (also used by tests).
fn list_env_vars_inner(cmd_or_alias: Option<&str>, settings: &Settings, ctx: OutputContext) {
    match cmd_or_alias {
        Some(name) => {
            let resolved = resolve_env_vars(settings, name);
            let resolved_from = settings.aliases.get(name).map(String::as_str);
            let views: Vec<EnvVarView<'_>> = resolved
                .iter()
                .map(|(k, v)| EnvVarView {
                    name: k.as_str(),
                    value: v.as_str(),
                })
                .collect();
            crate::output::render_list_env_vars(name, &views, resolved_from, ctx);
        },
        None => {
            for (group, vars) in &settings.env_vars {
                let views: Vec<EnvVarView<'_>> = vars
                    .iter()
                    .map(|(k, v)| EnvVarView {
                        name: k.as_str(),
                        value: v.as_str(),
                    })
                    .collect();
                crate::output::render_list_env_vars(group, &views, None, ctx);
            }
        },
    }
}

/// Build alias views from settings (helper to keep borrow lifetimes
/// clean).
fn alias_views_from_settings(settings: &Settings) -> Vec<AliasView<'_>> {
    settings
        .aliases
        .iter()
        .map(|(alias, target)| AliasView {
            alias: alias.as_str(),
            target: target.as_str(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Path-parameterized variants for testing
// ---------------------------------------------------------------------------

/// Same as [`list_aliases`] but loads settings from a specific path.
///
/// Exposed for tests so they can avoid mutating process-global env vars.
#[cfg(test)]
pub(super) fn list_aliases_from(
    ctx: OutputContext,
    settings_path: &std::path::Path,
) -> CliResult<()> {
    let settings = crate::settings::load_settings_from(settings_path)?;
    let views = alias_views_from_settings(&settings);
    crate::output::render_list_aliases(&views, ctx);
    Ok(())
}

/// Same as [`list_env_vars`] but loads settings from a specific path.
#[cfg(test)]
pub(super) fn list_env_vars_from(
    cmd_or_alias: Option<&str>,
    ctx: OutputContext,
    settings_path: &std::path::Path,
) -> CliResult<()> {
    let settings = crate::settings::load_settings_from(settings_path)?;
    list_env_vars_inner(cmd_or_alias, &settings, ctx);
    Ok(())
}

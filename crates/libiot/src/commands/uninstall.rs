//! Handler for the `uninstall` subcommand — wrapper around
//! `cargo uninstall`.

use std::process::Command;

use crate::cli::UninstallArgs;
use crate::cli::normalize_crate_name;
use crate::error::CliError;
use crate::error::CliResult;
use crate::output::CargoResultView;
use crate::output::OutputContext;
use crate::output::OutputFormat;
use crate::output::render_cargo_result;
use crate::settings::Settings;
use crate::settings::load_settings;
use crate::settings::save_settings;

/// Build the argument list for `cargo uninstall` from [`UninstallArgs`].
///
/// The returned vector does **not** include the leading
/// `cargo uninstall` tokens — only the flags and the crate name.
pub(crate) fn build_cargo_uninstall_args(
    crate_name: &str,
    args: &UninstallArgs,
    ctx: OutputContext,
) -> Vec<String> {
    let mut out = vec![crate_name.to_owned()];

    if let Some(ref root) = args.root {
        out.push("--root".to_owned());
        out.push(root.clone());
    }
    if ctx.verbose {
        out.push("--verbose".to_owned());
    }
    if args.quiet || ctx.quiet {
        out.push("--quiet".to_owned());
    }
    if let Some(ref color) = args.color {
        out.push("--color".to_owned());
        out.push(color.clone());
    }

    out
}

/// Execute the `uninstall` built-in command.
///
/// Wraps `cargo uninstall` with automatic crate-name prefixing and
/// optional cleanup of aliases and environment variables.
pub(crate) fn run_uninstall(args: &UninstallArgs, ctx: OutputContext) -> CliResult<()> {
    let short_name = normalize_crate_name(&args.name);
    let crate_name = format!("libiot-{short_name}-cli");
    let cargo_args = build_cargo_uninstall_args(&crate_name, args, ctx);

    // -- spawn cargo uninstall -------------------------------------------
    match ctx.format {
        OutputFormat::Human => {
            let status = Command::new("cargo")
                .arg("uninstall")
                .args(&cargo_args)
                .status()
                .map_err(|e| CliError::CargoSpawnFailed { source: e })?;

            if !status.success() {
                return Err(CliError::CargoUninstallFailed {
                    name: crate_name,
                    code: status.code().unwrap_or(1),
                });
            }
        },
        OutputFormat::Json => {
            let output = Command::new("cargo")
                .arg("uninstall")
                .args(&cargo_args)
                .output()
                .map_err(|e| CliError::CargoSpawnFailed { source: e })?;

            if !output.status.success() {
                let combined = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                );
                let view = CargoResultView {
                    ok: false,
                    crate_name: &crate_name,
                    cargo_output: Some(combined.trim()),
                    alias_created: None,
                };
                render_cargo_result(&view, ctx);
                return Err(CliError::CargoUninstallFailed {
                    name: crate_name,
                    code: output.status.code().unwrap_or(1),
                });
            }

            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
            let view = CargoResultView {
                ok: true,
                crate_name: &crate_name,
                cargo_output: Some(combined.trim()),
                alias_created: None,
            };
            render_cargo_result(&view, ctx);
        },
    }

    // -- post-uninstall cleanup ------------------------------------------
    // Aliases pointing to the uninstalled command are always removed
    // (stale aliases would break delegation). Env vars are only removed
    // with --remove-env-vars since they're harmless when the CLI is gone
    // and the user might reinstall later.
    {
        let mut settings = load_settings()?;
        cleanup_after_uninstall(
            &mut settings,
            short_name,
            /* remove_aliases = */ true,
            args.remove_env_vars,
        );
        save_settings(&settings)?;
    }

    if ctx.format == OutputFormat::Human {
        let view = CargoResultView {
            ok: true,
            crate_name: &crate_name,
            cargo_output: None,
            alias_created: None,
        };
        render_cargo_result(&view, ctx);
    }

    // -- regenerate completions ---------------------------------------------
    if !args.no_update_completions {
        crate::commands::completions::regenerate_existing_completions(ctx.verbose);
    }

    Ok(())
}

/// Remove aliases and/or env vars associated with a command.
///
/// When `remove_aliases` is true, any alias whose target equals `name`
/// is removed. When `remove_env_vars` is true, env vars keyed under
/// `name` (and under each removed alias) are removed.
///
/// This function mutates `settings` in place — the caller is
/// responsible for persisting the result.
pub(crate) fn cleanup_after_uninstall(
    settings: &mut Settings,
    name: &str,
    remove_aliases: bool,
    remove_env_vars: bool,
) {
    // Collect aliases that point to `name`.
    let matching_aliases: Vec<String> = settings
        .aliases
        .iter()
        .filter(|(_alias, target)| target.as_str() == name)
        .map(|(alias, _)| alias.clone())
        .collect();

    if remove_aliases {
        for alias in &matching_aliases {
            settings.aliases.remove(alias);
        }
    }

    if remove_env_vars {
        // Remove env vars for the command itself.
        settings.env_vars.remove(name);

        // Remove env vars for each alias that was (or would have been)
        // removed.
        for alias in &matching_aliases {
            settings.env_vars.remove(alias);
        }
    }
}

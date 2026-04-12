//! Handler for the `install` subcommand — wrapper around `cargo install`.

use std::process::Command;

use crate::cli::BUILTIN_NAMES;
use crate::cli::InstallArgs;
use crate::cli::normalize_crate_name;
use crate::error::CliError;
use crate::error::CliResult;
use crate::output::CargoResultView;
use crate::output::OutputContext;
use crate::output::OutputFormat;
use crate::output::render_cargo_result;
use crate::output::render_ok_message;
use crate::settings::load_settings;
use crate::settings::save_settings;

/// Build the argument list for `cargo install` from [`InstallArgs`].
///
/// The returned vector does **not** include the leading `cargo install`
/// tokens — only the flags and the crate name.
pub(crate) fn build_cargo_install_args(
    crate_name: &str,
    args: &InstallArgs,
    ctx_quiet: bool,
) -> Vec<String> {
    let mut out = vec![crate_name.to_owned()];

    if let Some(ref features) = args.features {
        out.push("--features".to_owned());
        out.push(features.clone());
    }
    if args.all_features {
        out.push("--all-features".to_owned());
    }
    if let Some(ref target_dir) = args.target_dir {
        out.push("--target-dir".to_owned());
        out.push(target_dir.clone());
    }
    if let Some(ref version) = args.version {
        out.push("--version".to_owned());
        out.push(version.clone());
    }
    if args.force {
        out.push("--force".to_owned());
    }
    if args.debug {
        out.push("--debug".to_owned());
    }
    if args.verbose {
        out.push("--verbose".to_owned());
    }
    if let Some(ref color) = args.color {
        out.push("--color".to_owned());
        out.push(color.clone());
    }
    if let Some(jobs) = args.jobs {
        out.push("--jobs".to_owned());
        out.push(jobs.to_string());
    }
    if args.quiet || ctx_quiet {
        out.push("--quiet".to_owned());
    }
    if let Some(ref root) = args.root {
        out.push("--root".to_owned());
        out.push(root.clone());
    }

    out
}

/// Execute the `install` built-in command.
///
/// Wraps `cargo install` with automatic crate-name prefixing and
/// optional post-install alias creation.
pub(crate) fn run_install(args: &InstallArgs, ctx: OutputContext) -> CliResult<()> {
    let short_name = normalize_crate_name(&args.name);
    let crate_name = format!("libiot-{short_name}-cli");
    let cargo_args = build_cargo_install_args(&crate_name, args, ctx.quiet);

    // -- dry run ---------------------------------------------------------
    if args.dry_run {
        let cmd_line = format!("cargo install {}", cargo_args.join(" "));
        render_ok_message(&format!("Dry run: {cmd_line}"), ctx);
        return Ok(());
    }

    // -- spawn cargo install ---------------------------------------------
    let alias_requested = args.alias.as_deref();
    let name_for_alias = short_name;

    match ctx.format {
        OutputFormat::Human => {
            let status = Command::new("cargo")
                .arg("install")
                .args(&cargo_args)
                .status()
                .map_err(|e| CliError::CargoSpawnFailed { source: e })?;

            if !status.success() {
                return Err(CliError::CargoInstallFailed {
                    name: crate_name,
                    code: status.code().unwrap_or(1),
                });
            }
        },
        OutputFormat::Json => {
            let output = Command::new("cargo")
                .arg("install")
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
                return Err(CliError::CargoInstallFailed {
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
                alias_created: alias_requested,
            };
            render_cargo_result(&view, ctx);
        },
    }

    // -- post-install alias ----------------------------------------------
    if let Some(alias_name) = alias_requested {
        create_post_install_alias(alias_name, name_for_alias)?;
    }

    if ctx.format == OutputFormat::Human {
        let view = CargoResultView {
            ok: true,
            crate_name: &crate_name,
            cargo_output: None,
            alias_created: alias_requested,
        };
        render_cargo_result(&view, ctx);
    }

    Ok(())
}

/// Create an alias after a successful install.
///
/// Always overwrites an existing alias (the user explicitly requested
/// `--alias`). Rejects names that shadow built-in commands.
fn create_post_install_alias(alias_name: &str, target_cmd: &str) -> CliResult<()> {
    if BUILTIN_NAMES.contains(&alias_name) {
        return Err(CliError::PostInstallAliasFailed {
            reason: format!("alias {alias_name:?} shadows built-in command {alias_name:?}"),
        });
    }

    let mut settings = load_settings().map_err(|e| CliError::PostInstallAliasFailed {
        reason: e.to_string(),
    })?;

    settings
        .aliases
        .insert(alias_name.to_owned(), target_cmd.to_owned());

    save_settings(&settings).map_err(|e| CliError::PostInstallAliasFailed {
        reason: e.to_string(),
    })?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Path-parameterized variant for testing
// ---------------------------------------------------------------------------

/// Same as [`create_post_install_alias`] but loads/saves from a specific
/// path. Used by tests to avoid mutating process-global state.
#[cfg(test)]
pub(super) fn create_post_install_alias_from(
    alias_name: &str,
    target_cmd: &str,
    settings_path: &std::path::Path,
) -> CliResult<()> {
    if BUILTIN_NAMES.contains(&alias_name) {
        return Err(CliError::PostInstallAliasFailed {
            reason: format!("alias {alias_name:?} shadows built-in command {alias_name:?}"),
        });
    }

    let mut settings = crate::settings::load_settings_from(settings_path).map_err(|e| {
        CliError::PostInstallAliasFailed {
            reason: e.to_string(),
        }
    })?;

    settings
        .aliases
        .insert(alias_name.to_owned(), target_cmd.to_owned());

    crate::settings::save_settings_to(&settings, settings_path).map_err(|e| {
        CliError::PostInstallAliasFailed {
            reason: e.to_string(),
        }
    })?;

    Ok(())
}

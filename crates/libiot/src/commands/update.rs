//! Handler for the `update` subcommand — update libiot or a specific
//! CLI to the latest version via `cargo install --force`.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::cli::UpdateArgs;
use crate::cli::normalize_crate_name;
use crate::error::CliError;
use crate::error::CliResult;
use crate::output::CargoResultView;
use crate::output::OutputContext;
use crate::output::OutputFormat;
use crate::output::render_cargo_result;
use crate::output::render_ok_message;

/// Execute the `update` built-in command.
///
/// With no name argument, updates `libiot` itself. With a name,
/// updates that specific CLI crate. In both cases, verifies the
/// target was installed via `cargo install` before proceeding.
pub(crate) fn run_update(args: &UpdateArgs, ctx: OutputContext) -> CliResult<()> {
    let (crate_name, is_self_update) = match args.name {
        Some(ref name) => {
            let short = normalize_crate_name(name);
            (format!("libiot-{short}-cli"), false)
        },
        None => ("libiot".to_owned(), true),
    };

    // -- verify cargo-installed ----------------------------------------------
    if !is_cargo_installed(&crate_name) {
        return Err(CliError::NotCargoInstalled { name: crate_name });
    }

    // -- build cargo args ----------------------------------------------------
    let cargo_args = build_cargo_update_args(&crate_name, args, ctx);

    // -- dry run -------------------------------------------------------------
    if args.dry_run {
        let cmd_line = format!("cargo install {}", cargo_args.join(" "));
        render_ok_message(&format!("Dry run: {cmd_line}"), ctx);
        return Ok(());
    }

    // -- spawn cargo install --force -----------------------------------------
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
                alias_created: None,
            };
            render_cargo_result(&view, ctx);
        },
    }

    // -- success output ------------------------------------------------------
    if ctx.format == OutputFormat::Human {
        let view = CargoResultView {
            ok: true,
            crate_name: &crate_name,
            cargo_output: None,
            alias_created: None,
        };
        render_cargo_result(&view, ctx);
    }

    // -- self-update hint ----------------------------------------------------
    if is_self_update && !ctx.quiet {
        eprintln!();
        eprintln!(
            "Note: libiot has been updated. Restart your shell or run \
             `source` on your shell config to pick up any completion changes."
        );
    }

    // -- regenerate completions ----------------------------------------------
    if !args.no_update_completions {
        crate::commands::completions::regenerate_existing_completions(ctx.verbose);
    }

    Ok(())
}

/// Build the argument list for `cargo install --force`.
fn build_cargo_update_args(crate_name: &str, args: &UpdateArgs, ctx: OutputContext) -> Vec<String> {
    let mut out = vec!["--force".to_owned(), crate_name.to_owned()];

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
    if args.debug {
        out.push("--debug".to_owned());
    }
    if ctx.verbose {
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
    if args.quiet || ctx.quiet {
        out.push("--quiet".to_owned());
    }
    if let Some(ref root) = args.root {
        out.push("--root".to_owned());
        out.push(root.clone());
    }

    out
}

/// Check whether a crate was installed via `cargo install` by reading
/// `~/.cargo/.crates2.json`.
///
/// Returns `true` if the crate appears in the install metadata,
/// `false` if the file is missing, unreadable, or doesn't list the
/// crate.
fn is_cargo_installed(crate_name: &str) -> bool {
    let Some(path) = cargo_crates_json_path() else {
        return false;
    };

    let Ok(contents) = fs::read_to_string(&path) else {
        return false;
    };

    let parsed: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let Some(installs) = parsed.get("installs").and_then(|v| v.as_object()) else {
        return false;
    };

    // Keys are formatted as "crate_name version (source)"
    installs
        .keys()
        .any(|key| key.starts_with(&format!("{crate_name} ")))
}

/// Path to cargo's installed-crates metadata file.
fn cargo_crates_json_path() -> Option<PathBuf> {
    let home = std::env::var("CARGO_HOME")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{h}/.cargo")))
        .ok()?;
    Some(PathBuf::from(home).join(".crates2.json"))
}

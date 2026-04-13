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
/// - No name, no `--all`: updates `libiot` itself.
/// - With a name: updates that specific CLI crate.
/// - With `--all`: updates every cargo-installed `libiot*` crate.
pub(crate) fn run_update(args: &UpdateArgs, ctx: OutputContext) -> CliResult<()> {
    if args.all {
        return run_update_all(args, ctx);
    }

    let (crate_name, is_self_update) = match args.name {
        Some(ref name) => {
            let short = normalize_crate_name(name);
            (format!("libiot-{short}-cli"), false)
        },
        None => ("libiot".to_owned(), true),
    };

    update_single_crate(&crate_name, args, ctx)?;

    if is_self_update && !ctx.quiet {
        print_self_update_hint();
    }

    if !args.no_update_completions {
        crate::commands::completions::regenerate_existing_completions(ctx.verbose);
    }

    Ok(())
}

/// Update every cargo-installed libiot crate (CLIs + libiot itself).
fn run_update_all(args: &UpdateArgs, ctx: OutputContext) -> CliResult<()> {
    let crates = list_installed_libiot_crates();

    if crates.is_empty() {
        if !ctx.quiet {
            eprintln!("No cargo-installed libiot crates found.");
        }
        return Ok(());
    }

    if !ctx.quiet {
        eprintln!(
            "Updating {} crate{}...",
            crates.len(),
            if crates.len() == 1 { "" } else { "s" }
        );
        eprintln!();
    }

    let mut any_failed = false;
    let mut updated_self = false;

    for crate_name in &crates {
        if !ctx.quiet {
            eprintln!("--- Updating {crate_name} ---");
        }

        match update_single_crate(crate_name, args, ctx) {
            Ok(()) => {
                if crate_name == "libiot" {
                    updated_self = true;
                }
            },
            Err(e) => {
                // Print the error but continue with remaining crates.
                eprintln!("error: {e}");
                eprintln!();
                any_failed = true;
            },
        }
    }

    if updated_self && !ctx.quiet {
        print_self_update_hint();
    }

    if !args.no_update_completions {
        crate::commands::completions::regenerate_existing_completions(ctx.verbose);
    }

    if any_failed {
        // Return a generic error so the exit code is non-zero, but
        // don't duplicate error messages — they were already printed.
        Err(CliError::CargoInstallFailed {
            name: "(some crates failed to update)".to_owned(),
            code: 1,
        })
    } else {
        Ok(())
    }
}

/// Update a single crate via `cargo install --force`.
///
/// Verifies the crate was cargo-installed, builds args, runs cargo,
/// and renders output. Does NOT regenerate completions or print the
/// self-update hint — the caller handles those.
fn update_single_crate(crate_name: &str, args: &UpdateArgs, ctx: OutputContext) -> CliResult<()> {
    if !is_cargo_installed(crate_name) {
        return Err(CliError::NotCargoInstalled {
            name: crate_name.to_owned(),
        });
    }

    let cargo_args = build_cargo_update_args(crate_name, args, ctx);

    if args.dry_run {
        let cmd_line = format!("cargo install {}", cargo_args.join(" "));
        render_ok_message(&format!("Dry run: {cmd_line}"), ctx);
        return Ok(());
    }

    match ctx.format {
        OutputFormat::Human => {
            let status = Command::new("cargo")
                .arg("install")
                .args(&cargo_args)
                .status()
                .map_err(|e| CliError::CargoSpawnFailed { source: e })?;

            if !status.success() {
                return Err(CliError::CargoInstallFailed {
                    name: crate_name.to_owned(),
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
                    crate_name,
                    cargo_output: Some(combined.trim()),
                    alias_created: None,
                };
                render_cargo_result(&view, ctx);
                return Err(CliError::CargoInstallFailed {
                    name: crate_name.to_owned(),
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
                crate_name,
                cargo_output: Some(combined.trim()),
                alias_created: None,
            };
            render_cargo_result(&view, ctx);
        },
    }

    if ctx.format == OutputFormat::Human {
        let view = CargoResultView {
            ok: true,
            crate_name,
            cargo_output: None,
            alias_created: None,
        };
        render_cargo_result(&view, ctx);
    }

    Ok(())
}

/// Print a hint that the user should reload their shell after
/// updating libiot itself.
fn print_self_update_hint() {
    eprintln!();
    eprintln!(
        "Note: libiot has been updated. Restart your shell or run \
         `source` on your shell config to pick up any completion changes."
    );
}

// ---------------------------------------------------------------------------
// Cargo arg building
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Cargo install metadata
// ---------------------------------------------------------------------------

/// Check whether a crate was installed via `cargo install` by reading
/// `~/.cargo/.crates2.json`.
fn is_cargo_installed(crate_name: &str) -> bool {
    installed_libiot_crate_names()
        .iter()
        .any(|name| name == crate_name)
}

/// List all cargo-installed crates whose name starts with `libiot`.
///
/// Returns crate names sorted alphabetically: `libiot` first (if
/// present), then `libiot-*-cli` crates.
fn list_installed_libiot_crates() -> Vec<String> {
    installed_libiot_crate_names()
}

/// Read `~/.cargo/.crates2.json` and return all installed crate names
/// that start with `libiot`.
fn installed_libiot_crate_names() -> Vec<String> {
    let Some(path) = cargo_crates_json_path() else {
        return Vec::new();
    };

    let Ok(contents) = fs::read_to_string(&path) else {
        return Vec::new();
    };

    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return Vec::new();
    };

    let Some(installs) = parsed.get("installs").and_then(|v| v.as_object()) else {
        return Vec::new();
    };

    let mut names: Vec<String> = installs
        .keys()
        .filter_map(|key| {
            // Keys are "crate_name version (source)"
            let crate_name = key.split(' ').next()?;
            if crate_name == "libiot" || crate_name.starts_with("libiot-") {
                Some(crate_name.to_owned())
            } else {
                None
            }
        })
        .collect();

    names.sort();
    names.dedup();
    names
}

/// Path to cargo's installed-crates metadata file.
fn cargo_crates_json_path() -> Option<PathBuf> {
    let home = std::env::var("CARGO_HOME")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{h}/.cargo")))
        .ok()?;
    Some(PathBuf::from(home).join(".crates2.json"))
}

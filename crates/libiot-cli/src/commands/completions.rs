//! Shell completion script generation.
//!
//! Generates completion scripts that include both built-in subcommands
//! and dynamically discovered `libiot-*-cli` binaries plus any
//! user-configured aliases.  The generated script is written to
//! `~/.config/libiot/completions/<shell>` and a short source snippet
//! is printed for the user to append to their shell configuration.

use std::fs;
use std::path::Path;

use clap::CommandFactory;

use crate::cli::Cli;
use crate::discovery::discover_clis;
use crate::error::CliError;
use crate::error::CliResult;
use crate::output::OutputContext;
use crate::settings::config_dir;
use crate::settings::load_settings;

/// Entry point for the `completions` subcommand.
///
/// - No shell argument: print setup instructions.
/// - Shell argument: write completions to disk and print a snippet.
/// - `--print-config` + shell: print *only* the snippet (for piping
///   into a shell config file).
pub(crate) fn run_completions(
    shell: Option<clap_complete::Shell>,
    print_config: bool,
    ctx: OutputContext,
) -> CliResult<()> {
    match (shell, print_config) {
        (Some(sh), true) => print_config_snippet(sh),
        (Some(sh), false) => write_and_print_snippet(sh, ctx),
        (None, _) => {
            print_install_instructions();
            Ok(())
        },
    }
}

// ---------------------------------------------------------------------------
// --print-config: snippet only
// ---------------------------------------------------------------------------

/// Print just the raw source snippet to stdout (no surrounding
/// instructions). Intended for `>> ~/.zshrc` style piping.
fn print_config_snippet(shell: clap_complete::Shell) -> CliResult<()> {
    let comp_dir = config_dir()?.join("completions");
    let file_path = comp_dir.join(completion_filename(shell));
    println!("{}", source_snippet(shell, &file_path));
    Ok(())
}

// ---------------------------------------------------------------------------
// Default: write file + print instructions
// ---------------------------------------------------------------------------

/// Generate the completion script, write it to
/// `~/.config/libiot/completions/<filename>`, and print a snippet plus
/// a one-liner alternative using `--print-config`.
fn write_and_print_snippet(shell: clap_complete::Shell, ctx: OutputContext) -> CliResult<()> {
    let script = generate_completions(shell);
    let comp_dir = config_dir()?.join("completions");
    let filename = completion_filename(shell);
    let file_path = comp_dir.join(filename);

    write_completion_file(&comp_dir, &file_path, &script)?;

    if !ctx.quiet {
        let snippet = source_snippet(shell, &file_path);
        let indented_snippet = indent(&snippet, "  ");
        let shell_name = completion_filename(shell);
        let config_file = shell_config_file(shell);

        println!("Completions written to {}", file_path.display());
        println!();
        println!("Add the following to your shell configuration file:");
        println!();
        println!("{indented_snippet}");
        println!();
        println!("Or run:");
        println!();
        println!("  libiot completions --print-config {shell_name} >> {config_file}");
        println!();
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The filename used inside the completions directory for each shell.
fn completion_filename(shell: clap_complete::Shell) -> &'static str {
    use clap_complete::Shell;
    match shell {
        Shell::Bash => "bash",
        Shell::Zsh => "zsh",
        Shell::Fish => "fish",
        Shell::PowerShell => "powershell",
        Shell::Elvish => "elvish",
        _ => "completions",
    }
}

/// The conventional shell configuration file for each shell.
fn shell_config_file(shell: clap_complete::Shell) -> &'static str {
    use clap_complete::Shell;
    match shell {
        Shell::Bash => "~/.bashrc",
        Shell::Zsh => "~/.zshrc",
        Shell::Fish => "~/.config/fish/config.fish",
        Shell::PowerShell => "$PROFILE",
        Shell::Elvish => "~/.elvish/rc.elv",
        _ => "<your shell config>",
    }
}

/// Build the snippet the user should put in their shell config.
fn source_snippet(shell: clap_complete::Shell, file_path: &Path) -> String {
    use clap_complete::Shell;
    let path = file_path.display();
    match shell {
        Shell::Bash | Shell::Zsh => format!(
            "# libiot tab-completion\n\
             if [ -f \"{path}\" ]; then\n\
             \x20 source \"{path}\"\n\
             fi"
        ),
        Shell::Fish => format!(
            "# libiot tab-completion\n\
             if test -f \"{path}\"\n\
             \x20 source \"{path}\"\n\
             end"
        ),
        Shell::PowerShell => format!(
            "# libiot tab-completion\n\
             if (Test-Path \"{path}\") {{ . \"{path}\" }}"
        ),
        Shell::Elvish => format!(
            "# libiot tab-completion\n\
             if (path:is-regular \"{path}\") {{\n\
             \x20 eval (slurp < \"{path}\")\n\
             }}"
        ),
        _ => format!("source \"{path}\""),
    }
}

/// Indent every line of `text` with `prefix`.
fn indent(text: &str, prefix: &str) -> String {
    text.lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Create the completions directory (if needed) and write the script.
fn write_completion_file(comp_dir: &Path, file_path: &Path, script: &str) -> CliResult<()> {
    fs::create_dir_all(comp_dir).map_err(|e| CliError::SettingsDirError {
        path: comp_dir.display().to_string(),
        source: e,
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(comp_dir, fs::Permissions::from_mode(0o700)).map_err(|e| {
            CliError::SettingsPermissionError {
                path: comp_dir.display().to_string(),
                source: e,
            }
        })?;
    }

    fs::write(file_path, script.as_bytes()).map_err(|e| CliError::SettingsWriteError {
        path: file_path.display().to_string(),
        source: e,
    })?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Instructions (no shell argument)
// ---------------------------------------------------------------------------

/// Print instructions for installing shell completions for each
/// supported shell.
fn print_install_instructions() {
    println!("Generate and install shell completions for libiot.");
    println!();
    println!("Usage: libiot completions <SHELL>");
    println!();
    println!("Supported shells: bash, zsh, fish, powershell, elvish");
    println!();
    println!("Running `libiot completions <SHELL>` will:");
    println!("  1. Write the completion script to ~/.config/libiot/completions/<SHELL>");
    println!("  2. Print a short snippet to add to your shell configuration file");
    println!();
    println!("Re-run after installing or uninstalling CLIs to keep completions");
    println!("up to date with newly discovered commands and aliases.");
}

// ---------------------------------------------------------------------------
// Completion generation core
// ---------------------------------------------------------------------------

/// Map a filename back to a [`clap_complete::Shell`].
///
/// Returns `None` if the filename doesn't match any known shell.
fn shell_from_filename(name: &str) -> Option<clap_complete::Shell> {
    use clap_complete::Shell;
    match name {
        "bash" => Some(Shell::Bash),
        "zsh" => Some(Shell::Zsh),
        "fish" => Some(Shell::Fish),
        "powershell" => Some(Shell::PowerShell),
        "elvish" => Some(Shell::Elvish),
        _ => None,
    }
}

/// Regenerate all completion files that already exist on disk.
///
/// Scans the completions directory for files matching known shell
/// names and regenerates each one. In verbose mode, prints progress
/// to stderr. Errors are printed to stderr but never cause a non-zero
/// exit.
fn regenerate_existing_completions_sync(verbose: bool) {
    let comp_dir = match config_dir() {
        Ok(d) => d.join("completions"),
        Err(e) => {
            if verbose {
                eprintln!("completions: could not resolve config dir: {e}");
            }
            return;
        },
    };

    if !comp_dir.is_dir() {
        if verbose {
            eprintln!(
                "completions: no completions directory at {}, skipping",
                comp_dir.display()
            );
        }
        return;
    }

    let entries = match fs::read_dir(&comp_dir) {
        Ok(e) => e,
        Err(e) => {
            if verbose {
                eprintln!("completions: could not read {}: {e}", comp_dir.display());
            }
            return;
        },
    };

    for entry in entries {
        let Ok(entry) = entry else { continue };
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let Some(shell) = shell_from_filename(&name_str) else {
            continue;
        };

        if verbose {
            eprintln!("completions: regenerating {name_str} completions...");
        }

        let script = generate_completions(shell);
        let file_path = comp_dir.join(&*name_str);
        if let Err(e) = fs::write(&file_path, script.as_bytes()) {
            if verbose {
                eprintln!("completions: failed to write {}: {e}", file_path.display());
            }
        } else if verbose {
            eprintln!("completions: wrote {}", file_path.display());
        }
    }
}

/// Regenerate existing completion files after install/uninstall.
///
/// - In **verbose** mode: runs in-process with progress output to
///   stderr. Errors are printed but do not affect the exit code.
/// - In **non-verbose** mode: forks a detached background process so
///   the file writes cannot block the main process from exiting.
///   Errors are silently swallowed.
pub(crate) fn regenerate_existing_completions(verbose: bool) {
    if verbose {
        regenerate_existing_completions_sync(/* verbose = */ true);
    } else {
        spawn_background_regeneration();
    }
}

/// Fork a detached child process that regenerates completions.
///
/// The child inherits `LIBIOT_CONFIG_DIR` (if set) so it writes to the
/// same location. Stdout and stderr are silenced. The parent does not
/// wait for the child.
fn spawn_background_regeneration() {
    // Get the path to our own binary so the child runs the same
    // executable.
    let Ok(exe) = std::env::current_exe() else {
        return;
    };

    // We can't directly call our own internal function in a fork
    // (Rust doesn't support fork safely with its runtime). Instead,
    // spawn ourselves as a child with a hidden subcommand-like
    // approach. But we don't have a hidden subcommand for this.
    //
    // Simpler approach: use std::process::Command to run a short
    // inline script. But we want to avoid shell injection.
    //
    // Simplest safe approach: invoke `libiot completions <SHELL>`
    // for each shell file that exists. But that would also print
    // the snippet output.
    //
    // Best approach: spawn a child that runs our binary with a
    // hidden environment variable signaling "regenerate mode".
    let _ = std::process::Command::new(exe)
        .env("_LIBIOT_REGEN_COMPLETIONS", "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    // Parent does not wait. Child is detached.
}

/// Entry point for the background regeneration child process.
///
/// Called from `main()` when `_LIBIOT_REGEN_COMPLETIONS=1` is set.
/// Regenerates all existing completion files, then exits.
pub(crate) fn run_background_regeneration() -> ! {
    regenerate_existing_completions_sync(/* verbose = */ false);
    std::process::exit(0)
}

/// Generate shell completions and return the output as a [`String`].
///
/// This is the testable core; it writes into an in-memory buffer
/// instead of a file or stdout.
pub(crate) fn generate_completions(shell: clap_complete::Shell) -> String {
    let mut cmd = Cli::command();

    let discovered = discover_clis();
    let discovered_names: std::collections::BTreeSet<String> =
        discovered.iter().map(|c| c.name.clone()).collect();

    for name in &discovered_names {
        let leaked: &'static str = Box::leak(name.clone().into_boxed_str());
        cmd = cmd.subcommand(clap::Command::new(leaked).hide(true).trailing_var_arg(true));
    }

    if let Ok(settings) = load_settings() {
        for alias_name in settings.aliases.keys() {
            if !discovered_names.contains(alias_name) {
                let leaked: &'static str = Box::leak(alias_name.clone().into_boxed_str());
                cmd = cmd.subcommand(clap::Command::new(leaked).hide(true).trailing_var_arg(true));
            }
        }
    }

    let mut buf: Vec<u8> = Vec::new();
    clap_complete::generate(shell, &mut cmd, "libiot", &mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}

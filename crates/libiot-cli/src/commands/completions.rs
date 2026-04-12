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

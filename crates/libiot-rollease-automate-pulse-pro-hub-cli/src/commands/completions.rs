//! Shell completion and man page generation.
//!
//! Generates completion scripts and writes them to
//! `~/.config/libiot/completions/libiot-rollease-automate-pulse-pro-hub/<shell>`.
//! When invoked without a shell argument, prints installation
//! instructions listing supported shells.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use clap::CommandFactory;

use crate::cli::Cli;

/// The binary name used for completion generation.
const BIN_NAME: &str = "libiot-rollease-automate-pulse-pro-hub";

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Entry point for the `completions` subcommand.
///
/// - No shell argument: print setup instructions.
/// - Shell argument: write completions to disk and print a snippet.
/// - `--print-config` + shell: print *only* the snippet (for piping
///   into a shell config file).
pub(crate) fn run_completions(shell: Option<clap_complete::Shell>, print_config: bool) {
    match (shell, print_config) {
        (Some(sh), true) => print_config_snippet(sh),
        (Some(sh), false) => write_and_print_snippet(sh),
        (None, _) => print_install_instructions(),
    }
}

/// Write a man page to `path`.
pub(crate) fn run_man_page(path: &Path) {
    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);
    let mut out = std::fs::File::create(path).unwrap_or_else(|err| {
        eprintln!("error: could not create {}: {err}", path.display());
        std::process::exit(1);
    });
    man.render(&mut out).unwrap_or_else(|err| {
        eprintln!(
            "error: could not write man page to {}: {err}",
            path.display()
        );
        std::process::exit(1);
    });
    eprintln!("man page written to {}", path.display());
}

// ---------------------------------------------------------------------------
// --print-config: snippet only
// ---------------------------------------------------------------------------

/// Print just the raw source snippet to stdout (no surrounding
/// instructions). Intended for `>> ~/.zshrc` style piping.
fn print_config_snippet(shell: clap_complete::Shell) {
    let comp_dir = completions_dir();
    let file_path = comp_dir.join(completion_filename(shell));
    println!("{}", source_snippet(shell, &file_path));
}

// ---------------------------------------------------------------------------
// Default: write file + print instructions
// ---------------------------------------------------------------------------

/// Generate the completion script, write it to
/// `~/.config/libiot/completions/libiot-rollease-automate-pulse-pro-hub/<filename>`,
/// and print a snippet plus a one-liner alternative using
/// `--print-config`.
fn write_and_print_snippet(shell: clap_complete::Shell) {
    let mut cmd = Cli::command();
    let mut buf: Vec<u8> = Vec::new();
    clap_complete::generate(shell, &mut cmd, BIN_NAME, &mut buf);
    let script = String::from_utf8_lossy(&buf).into_owned();

    let comp_dir = completions_dir();
    let filename = completion_filename(shell);
    let file_path = comp_dir.join(filename);

    write_completion_file(&comp_dir, &file_path, &script);

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
    println!(
        "  {BIN_NAME} completions --print-config {shell_name} \
         >> {config_file}"
    );
    println!();
}

// ---------------------------------------------------------------------------
// Instructions (no shell argument)
// ---------------------------------------------------------------------------

/// Print instructions for installing shell completions for each
/// supported shell.
fn print_install_instructions() {
    println!("Generate and install shell completions for {BIN_NAME}.");
    println!();
    println!("Usage: {BIN_NAME} completions <SHELL>");
    println!();
    println!("Supported shells: bash, zsh, fish, powershell, elvish");
    println!();
    println!("Running `{BIN_NAME} completions <SHELL>` will:");
    println!(
        "  1. Write the completion script to \
         ~/.config/libiot/completions/{BIN_NAME}/<SHELL>"
    );
    println!(
        "  2. Print a short snippet to add to your shell \
         configuration file"
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the completions directory for this binary.
///
/// Resolves `LIBIOT_CONFIG_DIR` (if set), falling back to
/// `$HOME/.config/libiot`, then appends
/// `completions/libiot-rollease-automate-pulse-pro-hub`.
fn completions_dir() -> PathBuf {
    config_dir().join("completions").join(BIN_NAME)
}

/// Return the libiot configuration directory.
///
/// Checks `LIBIOT_CONFIG_DIR` first; falls back to
/// `$HOME/.config/libiot`.
fn config_dir() -> PathBuf {
    if let Ok(d) = std::env::var("LIBIOT_CONFIG_DIR") {
        if !d.is_empty() {
            return PathBuf::from(d);
        }
    }

    let home = match std::env::var("HOME") {
        Ok(h) if !h.is_empty() => h,
        _ => {
            eprintln!(
                "error: could not determine home directory \
                 (set LIBIOT_CONFIG_DIR or HOME)"
            );
            std::process::exit(1);
        },
    };

    PathBuf::from(home).join(".config").join("libiot")
}

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
            "# {BIN_NAME} tab-completion\n\
             if [ -f \"{path}\" ]; then\n\
             \x20 source \"{path}\"\n\
             fi"
        ),
        Shell::Fish => format!(
            "# {BIN_NAME} tab-completion\n\
             if test -f \"{path}\"\n\
             \x20 source \"{path}\"\n\
             end"
        ),
        Shell::PowerShell => format!(
            "# {BIN_NAME} tab-completion\n\
             if (Test-Path \"{path}\") {{ . \"{path}\" }}"
        ),
        Shell::Elvish => format!(
            "# {BIN_NAME} tab-completion\n\
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
///
/// Prints an error to stderr and exits with code 1 on failure.
fn write_completion_file(comp_dir: &Path, file_path: &Path, script: &str) {
    if let Err(e) = fs::create_dir_all(comp_dir) {
        eprintln!("error: could not create {}: {e}", comp_dir.display());
        std::process::exit(1);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = fs::set_permissions(comp_dir, fs::Permissions::from_mode(0o700)) {
            eprintln!(
                "error: could not set permissions on {}: {e}",
                comp_dir.display()
            );
            std::process::exit(1);
        }
    }

    if let Err(e) = fs::write(file_path, script.as_bytes()) {
        eprintln!("error: could not write {}: {e}", file_path.display());
        std::process::exit(1);
    }
}

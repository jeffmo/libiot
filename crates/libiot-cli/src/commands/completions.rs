//! Shell completion script generation.
//!
//! Generates completion scripts that include both built-in subcommands
//! and dynamically discovered `libiot-*-cli` binaries plus any
//! user-configured aliases.

use clap::CommandFactory;

use crate::cli::Cli;
use crate::discovery::discover_clis;
use crate::settings::load_settings;

/// Generate shell completions to stdout for the given shell, or print
/// installation instructions if no shell is specified.
///
/// Discovered CLI binaries and configured aliases are injected as hidden
/// subcommands so that shell completion can suggest them even though they
/// are not part of the static clap tree.
pub(crate) fn run_completions(shell: Option<clap_complete::Shell>) {
    match shell {
        Some(sh) => {
            let output = generate_completions(sh);
            print!("{output}");
        },
        None => print_install_instructions(),
    }
}

/// Print instructions for installing shell completions for each
/// supported shell.
fn print_install_instructions() {
    println!("Generate and install shell completions for libiot.");
    println!();
    println!("Usage: libiot completions <SHELL>");
    println!();
    println!("Supported shells and installation instructions:");
    println!();
    println!("  bash:");
    println!("    libiot completions bash > ~/.local/share/bash-completion/completions/libiot");
    println!();
    println!("  zsh:");
    println!("    libiot completions zsh > ~/.zfunc/_libiot");
    println!("    # Ensure ~/.zfunc is in your fpath (add to ~/.zshrc if needed):");
    println!("    #   fpath=(~/.zfunc $fpath)");
    println!("    #   autoload -Uz compinit && compinit");
    println!();
    println!("  fish:");
    println!("    libiot completions fish > ~/.config/fish/completions/libiot.fish");
    println!();
    println!("  powershell:");
    println!("    libiot completions powershell >> $PROFILE");
    println!();
    println!("  elvish:");
    println!("    libiot completions elvish >> ~/.elvish/rc.elv");
    println!();
    println!("Completions include both built-in commands and any libiot-*");
    println!("CLIs currently on PATH plus configured aliases. Re-run after");
    println!("installing or uninstalling CLIs to keep completions up to date.");
}

/// Generate shell completions and return the output as a [`String`].
///
/// This is the testable core of [`run_completions`]; it writes into an
/// in-memory buffer instead of stdout.
pub(crate) fn generate_completions(shell: clap_complete::Shell) -> String {
    let mut cmd = Cli::command();

    // Collect discovered CLI names.
    let discovered = discover_clis();
    let discovered_names: std::collections::BTreeSet<String> =
        discovered.iter().map(|c| c.name.clone()).collect();

    // Inject discovered CLIs as hidden subcommands.
    //
    // `clap::Command::new` requires `impl Into<Str>` which only accepts
    // `&'static str`.  Since this is a one-shot generation function we
    // leak the dynamic names — the process exits shortly after anyway.
    for name in &discovered_names {
        let leaked: &'static str = Box::leak(name.clone().into_boxed_str());
        cmd = cmd.subcommand(clap::Command::new(leaked).hide(true).trailing_var_arg(true));
    }

    // Inject aliases as hidden subcommands (skip names already added).
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

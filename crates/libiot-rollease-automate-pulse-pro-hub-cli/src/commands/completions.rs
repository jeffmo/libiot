//! Shell completion and man page generation.

use std::io;
use std::path::Path;

use clap::CommandFactory;

use crate::cli::Cli;

/// Generate shell completions to stdout for the given shell.
pub(crate) fn run_completions(shell: clap_complete::Shell) {
    let mut cmd = Cli::command();
    clap_complete::generate(
        shell,
        &mut cmd,
        "libiot-rollease-automate-pulse-pro-hub",
        &mut io::stdout(),
    );
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

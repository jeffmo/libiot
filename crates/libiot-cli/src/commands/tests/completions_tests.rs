//! Tests for the `completions` command handler.
//!
//! Written by Claude Code, reviewed by a human.

use crate::commands::completions::generate_completions;

/// Generating bash completions does not panic.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn completions_for_bash_does_not_panic() {
    let output = generate_completions(clap_complete::Shell::Bash);
    assert!(!output.is_empty(), "bash completions should produce output");
}

/// Generating zsh completions does not panic.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn completions_for_zsh_does_not_panic() {
    let output = generate_completions(clap_complete::Shell::Zsh);
    assert!(!output.is_empty(), "zsh completions should produce output");
}

/// Bash completion output contains references to built-in subcommands.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn completions_output_contains_builtin_commands() {
    let output = generate_completions(clap_complete::Shell::Bash);
    for name in &["set", "install", "list", "completions", "config-path"] {
        assert!(
            output.contains(name),
            "bash completions should mention the '{name}' subcommand"
        );
    }
}

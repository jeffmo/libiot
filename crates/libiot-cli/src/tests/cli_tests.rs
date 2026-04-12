//! Tests for [`crate::cli`] — built-in name detection and the
//! `BUILTIN_NAMES` constant.

use crate::cli::BUILTIN_NAMES;
use crate::cli::is_builtin;

/// `is_builtin("set")` returns true because `set` is a built-in
/// subcommand for alias/env-var management.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn is_builtin_set() {
    assert!(is_builtin("set"));
}

/// `is_builtin("get")` returns true because `get` is a built-in
/// subcommand for querying aliases and env vars.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn is_builtin_get() {
    assert!(is_builtin("get"));
}

/// `is_builtin("config-path")` returns true because `config-path` is
/// a built-in subcommand that prints the settings file location.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn is_builtin_config_path() {
    assert!(is_builtin("config-path"));
}

/// `is_builtin("help")` returns true because `help` is reserved as a
/// built-in subcommand.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn is_builtin_help() {
    assert!(is_builtin("help"));
}

/// `is_builtin("shades")` returns false because `shades` is not a
/// built-in subcommand — it would be delegated to a discovered CLI.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn is_builtin_shades_is_not_builtin() {
    assert!(!is_builtin("shades"));
}

/// `is_builtin("SET")` returns false because built-in name matching
/// is case-sensitive (all built-in names are lowercase).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn is_builtin_case_sensitive() {
    assert!(!is_builtin("SET"));
}

/// `BUILTIN_NAMES` is sorted in alphabetical order so that binary
/// search or ordered iteration assumptions hold.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn builtin_names_sorted() {
    let mut sorted = BUILTIN_NAMES.to_vec();
    sorted.sort();
    assert_eq!(BUILTIN_NAMES, sorted.as_slice());
}

//! Tests for the `uninstall` command handler.
//!
//! Written by Claude Code, reviewed by a human.

use std::collections::BTreeMap;

use crate::cli::UninstallArgs;
use crate::commands::uninstall::build_cargo_uninstall_args;
use crate::commands::uninstall::cleanup_after_uninstall;
use crate::settings::Settings;

/// Build default [`UninstallArgs`] with only the `name` field populated.
fn default_args(name: &str) -> UninstallArgs {
    UninstallArgs {
        name: name.to_owned(),
        color: None,
        quiet: false,
        remove_aliases: false,
        remove_env_vars: false,
        root: None,
        verbose: false,
        no_update_completions: true,
    }
}

/// Create a [`Settings`] with some aliases and env vars for testing
/// cleanup logic.
fn sample_settings() -> Settings {
    let mut settings = Settings::default();

    // Two aliases pointing to "rollease".
    settings
        .aliases
        .insert("shades".to_owned(), "rollease".to_owned());
    settings
        .aliases
        .insert("blinds".to_owned(), "rollease".to_owned());

    // One alias pointing to a different command.
    settings
        .aliases
        .insert("lights".to_owned(), "hue".to_owned());

    // Env vars for "rollease", "shades", "blinds", and "hue".
    settings.env_vars.insert(
        "rollease".to_owned(),
        BTreeMap::from([("HUB_IP".to_owned(), "10.0.0.1".to_owned())]),
    );
    settings.env_vars.insert(
        "shades".to_owned(),
        BTreeMap::from([("ROOM".to_owned(), "living".to_owned())]),
    );
    settings.env_vars.insert(
        "blinds".to_owned(),
        BTreeMap::from([("ROOM".to_owned(), "bedroom".to_owned())]),
    );
    settings.env_vars.insert(
        "hue".to_owned(),
        BTreeMap::from([("BRIDGE".to_owned(), "10.0.0.2".to_owned())]),
    );

    settings
}

// -----------------------------------------------------------------------
// build_cargo_uninstall_args tests
// -----------------------------------------------------------------------

/// Minimal args produce only the crate name.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn build_args_minimal() {
    let args = default_args("foo");
    let result = build_cargo_uninstall_args("libiot-foo-cli", &args, false);
    assert_eq!(result, vec!["libiot-foo-cli"]);
}

/// All optional flags are forwarded to cargo.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn build_args_all_flags() {
    let args = UninstallArgs {
        name: "test".to_owned(),
        color: Some("never".to_owned()),
        quiet: true,
        remove_aliases: false,
        remove_env_vars: false,
        root: Some("/usr/local".to_owned()),
        verbose: true,
        no_update_completions: true,
    };
    let result = build_cargo_uninstall_args("libiot-test-cli", &args, false);
    assert_eq!(
        result,
        vec![
            "libiot-test-cli",
            "--root",
            "/usr/local",
            "--verbose",
            "--quiet",
            "--color",
            "never",
        ],
    );
}

/// `ctx_quiet` adds `--quiet` even when the arg is not set.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn build_args_ctx_quiet_adds_quiet() {
    let args = default_args("foo");
    let result = build_cargo_uninstall_args("libiot-foo-cli", &args, true);
    assert!(result.contains(&"--quiet".to_owned()));
}

/// When both `args.quiet` and `ctx_quiet` are true, `--quiet` appears
/// exactly once.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn build_args_quiet_not_duplicated() {
    let mut args = default_args("foo");
    args.quiet = true;
    let result = build_cargo_uninstall_args("libiot-foo-cli", &args, true);
    let quiet_count = result.iter().filter(|a| a.as_str() == "--quiet").count();
    assert_eq!(quiet_count, 1, "expected exactly one --quiet flag");
}

// -----------------------------------------------------------------------
// cleanup_after_uninstall tests
// -----------------------------------------------------------------------

/// `--remove-aliases` removes only aliases pointing to the target
/// command.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn cleanup_remove_aliases_only() {
    let mut settings = sample_settings();
    cleanup_after_uninstall(&mut settings, "rollease", true, false);

    // "shades" and "blinds" should be gone.
    assert!(!settings.aliases.contains_key("shades"));
    assert!(!settings.aliases.contains_key("blinds"));

    // "lights" should remain.
    assert_eq!(
        settings.aliases.get("lights").map(String::as_str),
        Some("hue"),
    );

    // Env vars should be untouched.
    assert!(settings.env_vars.contains_key("rollease"));
    assert!(settings.env_vars.contains_key("shades"));
    assert!(settings.env_vars.contains_key("blinds"));
}

/// `--remove-env-vars` removes env vars for the command and its
/// aliases.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn cleanup_remove_env_vars_only() {
    let mut settings = sample_settings();
    cleanup_after_uninstall(&mut settings, "rollease", false, true);

    // Env vars for "rollease", "shades", and "blinds" should be gone.
    assert!(!settings.env_vars.contains_key("rollease"));
    assert!(!settings.env_vars.contains_key("shades"));
    assert!(!settings.env_vars.contains_key("blinds"));

    // Env vars for "hue" should remain.
    assert!(settings.env_vars.contains_key("hue"));

    // Aliases should be untouched.
    assert!(settings.aliases.contains_key("shades"));
    assert!(settings.aliases.contains_key("blinds"));
    assert!(settings.aliases.contains_key("lights"));
}

/// Both flags together remove aliases and env vars.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn cleanup_both_flags() {
    let mut settings = sample_settings();
    cleanup_after_uninstall(&mut settings, "rollease", true, true);

    // Aliases for "rollease" gone.
    assert!(!settings.aliases.contains_key("shades"));
    assert!(!settings.aliases.contains_key("blinds"));

    // Env vars for "rollease" and its aliases gone.
    assert!(!settings.env_vars.contains_key("rollease"));
    assert!(!settings.env_vars.contains_key("shades"));
    assert!(!settings.env_vars.contains_key("blinds"));

    // Unrelated command untouched.
    assert_eq!(
        settings.aliases.get("lights").map(String::as_str),
        Some("hue"),
    );
    assert!(settings.env_vars.contains_key("hue"));
}

/// Cleanup with no matching aliases is a no-op.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn cleanup_no_matching_aliases() {
    let mut settings = sample_settings();
    let original_aliases = settings.aliases.clone();
    let original_env_vars = settings.env_vars.clone();

    cleanup_after_uninstall(&mut settings, "nonexistent", true, true);

    assert_eq!(settings.aliases, original_aliases);
    assert_eq!(settings.env_vars, original_env_vars);
}

/// `--remove-env-vars` removes command env vars even when there are
/// no aliases for that command.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn cleanup_removes_command_env_vars_without_aliases() {
    let mut settings = Settings::default();
    settings.env_vars.insert(
        "standalone".to_owned(),
        BTreeMap::from([("KEY".to_owned(), "val".to_owned())]),
    );

    cleanup_after_uninstall(&mut settings, "standalone", false, true);

    assert!(!settings.env_vars.contains_key("standalone"));
}

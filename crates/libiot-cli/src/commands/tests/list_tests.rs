//! Tests for the `list` command handler.
//!
//! Written by Claude Code, reviewed by a human.

use std::collections::BTreeMap;

use crate::commands::list::list_aliases_from;
use crate::commands::list::list_env_vars_from;
use crate::output::OutputContext;
use crate::output::OutputFormat;
use crate::settings::Settings;
use crate::settings::save_settings_to;

/// Build an [`OutputContext`] with quiet mode enabled.
fn quiet_ctx() -> OutputContext {
    OutputContext {
        format: OutputFormat::Human,
        quiet: true,
    }
}

/// Create a temp dir with a settings file and return the path to
/// `settings.json`.
fn write_settings(settings: &Settings) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("create tempdir");
    let path = dir.path().join("settings.json");
    save_settings_to(settings, &path).expect("write settings");
    (dir, path)
}

/// `list aliases` with no aliases succeeds (shows empty output).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_aliases_empty_succeeds() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let result = list_aliases_from(quiet_ctx(), &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

/// `list aliases` with configured aliases succeeds.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_aliases_with_aliases_succeeds() {
    let mut settings = Settings::default();
    settings
        .aliases
        .insert("shades".to_owned(), "rollease".to_owned());
    settings
        .aliases
        .insert("lights".to_owned(), "hue-bridge".to_owned());
    let (_dir, path) = write_settings(&settings);

    let result = list_aliases_from(quiet_ctx(), &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

/// `list env-vars` for a specific command shows its variables.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_env_vars_for_command_succeeds() {
    let mut settings = Settings::default();
    let mut inner = BTreeMap::new();
    inner.insert("HUB_IP".to_owned(), "192.168.1.1".to_owned());
    inner.insert("TIMEOUT".to_owned(), "30".to_owned());
    settings.env_vars.insert("shades".to_owned(), inner);
    let (_dir, path) = write_settings(&settings);

    let result = list_env_vars_from(Some("shades"), quiet_ctx(), &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

/// `list env-vars` for an alias shows resolved/merged variables
/// (including those from the underlying command).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_env_vars_for_alias_shows_resolved_vars() {
    let mut settings = Settings::default();
    settings
        .aliases
        .insert("shades".to_owned(), "rollease".to_owned());

    // Set env vars on the underlying command.
    let mut cmd_vars = BTreeMap::new();
    cmd_vars.insert("HUB_IP".to_owned(), "192.168.1.1".to_owned());
    settings.env_vars.insert("rollease".to_owned(), cmd_vars);

    // Set an alias-specific env var that overlays the command vars.
    let mut alias_vars = BTreeMap::new();
    alias_vars.insert("TIMEOUT".to_owned(), "60".to_owned());
    settings.env_vars.insert("shades".to_owned(), alias_vars);
    let (_dir, path) = write_settings(&settings);

    let result = list_env_vars_from(Some("shades"), quiet_ctx(), &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

/// `list env-vars` without a specific command lists all groups.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_env_vars_all_groups_succeeds() {
    let mut settings = Settings::default();
    let mut inner1 = BTreeMap::new();
    inner1.insert("HUB_IP".to_owned(), "192.168.1.1".to_owned());
    settings.env_vars.insert("shades".to_owned(), inner1);

    let mut inner2 = BTreeMap::new();
    inner2.insert("BRIDGE_IP".to_owned(), "10.0.0.5".to_owned());
    settings.env_vars.insert("lights".to_owned(), inner2);
    let (_dir, path) = write_settings(&settings);

    let result = list_env_vars_from(None, quiet_ctx(), &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

/// `list env-vars` for a command with no env vars returns Ok with
/// empty output.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_env_vars_for_command_no_vars_succeeds() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let result = list_env_vars_from(Some("nonexistent"), quiet_ctx(), &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

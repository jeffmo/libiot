//! Tests for the `unset` command handler.
//!
//! Written by Claude Code, reviewed by a human.

use std::collections::BTreeMap;

use crate::cli::UnsetTarget;
use crate::commands::unset::run_unset_from;
use crate::error::CliError;
use crate::output::OutputContext;
use crate::output::OutputFormat;
use crate::settings::Settings;
use crate::settings::load_settings_from;
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

/// `unset alias` for an existing alias removes it from settings.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unset_alias_existing_removes_it() {
    let mut settings = Settings::default();
    settings
        .aliases
        .insert("shades".to_owned(), "rollease".to_owned());
    let (_dir, path) = write_settings(&settings);

    let target = UnsetTarget::Alias {
        alias_name: "shades".to_owned(),
    };
    let result = run_unset_from(target, quiet_ctx(), &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");

    let reloaded = load_settings_from(&path).expect("reload settings");
    assert!(reloaded.aliases.is_empty());
}

/// `unset alias` for a missing alias returns `AliasNotFound`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unset_alias_missing_returns_alias_not_found() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let target = UnsetTarget::Alias {
        alias_name: "nonexistent".to_owned(),
    };
    let err = run_unset_from(target, quiet_ctx(), &path).unwrap_err();
    assert!(
        matches!(err, CliError::AliasNotFound { ref alias } if alias == "nonexistent"),
        "expected AliasNotFound, got {err:?}",
    );
}

/// `unset env-var` for an existing variable removes it from settings.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unset_env_var_existing_removes_it() {
    let mut settings = Settings::default();
    let mut inner = BTreeMap::new();
    inner.insert("HUB_IP".to_owned(), "192.168.1.1".to_owned());
    inner.insert("TIMEOUT".to_owned(), "30".to_owned());
    settings.env_vars.insert("shades".to_owned(), inner);
    let (_dir, path) = write_settings(&settings);

    let target = UnsetTarget::EnvVar {
        cmd_or_alias: "shades".to_owned(),
        var_name: "HUB_IP".to_owned(),
    };
    let result = run_unset_from(target, quiet_ctx(), &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");

    let reloaded = load_settings_from(&path).expect("reload settings");
    let remaining = reloaded.env_vars.get("shades");
    assert!(remaining.is_some(), "outer key should still exist");
    assert!(!remaining.unwrap().contains_key("HUB_IP"));
    assert!(remaining.unwrap().contains_key("TIMEOUT"));
}

/// `unset env-var` for a missing variable returns `EnvVarNotFound`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unset_env_var_missing_returns_env_var_not_found() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let target = UnsetTarget::EnvVar {
        cmd_or_alias: "shades".to_owned(),
        var_name: "MISSING".to_owned(),
    };
    let err = run_unset_from(target, quiet_ctx(), &path).unwrap_err();
    assert!(
        matches!(
            err,
            CliError::EnvVarNotFound {
                ref cmd_or_alias,
                ref name,
            } if cmd_or_alias == "shades" && name == "MISSING"
        ),
        "expected EnvVarNotFound, got {err:?}",
    );
}

/// `unset env-var` cleans up the outer key when the inner map becomes
/// empty.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unset_env_var_cleans_up_empty_inner_map() {
    let mut settings = Settings::default();
    let mut inner = BTreeMap::new();
    inner.insert("HUB_IP".to_owned(), "192.168.1.1".to_owned());
    settings.env_vars.insert("shades".to_owned(), inner);
    let (_dir, path) = write_settings(&settings);

    let target = UnsetTarget::EnvVar {
        cmd_or_alias: "shades".to_owned(),
        var_name: "HUB_IP".to_owned(),
    };
    let result = run_unset_from(target, quiet_ctx(), &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");

    let reloaded = load_settings_from(&path).expect("reload settings");
    assert!(
        reloaded.env_vars.is_empty(),
        "outer key should be removed when inner map is empty",
    );
}

/// `unset env-var` when the outer key exists but the inner key does
/// not returns `EnvVarNotFound`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unset_env_var_inner_missing_returns_env_var_not_found() {
    let mut settings = Settings::default();
    let mut inner = BTreeMap::new();
    inner.insert("HUB_IP".to_owned(), "192.168.1.1".to_owned());
    settings.env_vars.insert("shades".to_owned(), inner);
    let (_dir, path) = write_settings(&settings);

    let target = UnsetTarget::EnvVar {
        cmd_or_alias: "shades".to_owned(),
        var_name: "WRONG_KEY".to_owned(),
    };
    let err = run_unset_from(target, quiet_ctx(), &path).unwrap_err();
    assert!(
        matches!(
            err,
            CliError::EnvVarNotFound {
                ref cmd_or_alias,
                ref name,
            } if cmd_or_alias == "shades" && name == "WRONG_KEY"
        ),
        "expected EnvVarNotFound, got {err:?}",
    );
}

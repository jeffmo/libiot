//! Tests for the `get` command handler.
//!
//! Written by Claude Code, reviewed by a human.

use std::collections::BTreeMap;

use crate::cli::GetTarget;
use crate::commands::get::run_get_from;
use crate::error::CliError;
use crate::output::OutputContext;
use crate::output::OutputFormat;
use crate::settings::Settings;
use crate::settings::save_settings_to;

/// Build an [`OutputContext`] with quiet mode enabled (suppresses
/// stdout so tests don't produce spurious output).
fn quiet_ctx() -> OutputContext {
    OutputContext {
        format: OutputFormat::Human,
        quiet: true,
        verbose: false,
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

/// `get alias` for an existing alias returns `Ok`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn get_alias_existing_returns_ok() {
    let mut settings = Settings::default();
    settings
        .aliases
        .insert("shades".to_owned(), "rollease".to_owned());
    let (_dir, path) = write_settings(&settings);

    let target = GetTarget::Alias {
        alias_name: "shades".to_owned(),
    };
    let result = run_get_from(target, quiet_ctx(), &path);
    assert!(result.is_ok());
}

/// `get alias` for a missing alias returns `AliasNotFound`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn get_alias_missing_returns_alias_not_found() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let target = GetTarget::Alias {
        alias_name: "nonexistent".to_owned(),
    };
    let err = run_get_from(target, quiet_ctx(), &path).unwrap_err();
    assert!(
        matches!(err, CliError::AliasNotFound { ref alias } if alias == "nonexistent"),
        "expected AliasNotFound, got {err:?}",
    );
}

/// `get env-var` for an existing variable returns `Ok`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn get_env_var_existing_returns_ok() {
    let mut settings = Settings::default();
    let mut inner = BTreeMap::new();
    inner.insert("HUB_IP".to_owned(), "192.168.1.1".to_owned());
    settings.env_vars.insert("shades".to_owned(), inner);
    let (_dir, path) = write_settings(&settings);

    let target = GetTarget::EnvVar {
        cmd_or_alias: "shades".to_owned(),
        var_name: "HUB_IP".to_owned(),
    };
    let result = run_get_from(target, quiet_ctx(), &path);
    assert!(result.is_ok());
}

/// `get env-var` for a missing variable returns `EnvVarNotFound`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn get_env_var_missing_returns_env_var_not_found() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let target = GetTarget::EnvVar {
        cmd_or_alias: "shades".to_owned(),
        var_name: "MISSING".to_owned(),
    };
    let err = run_get_from(target, quiet_ctx(), &path).unwrap_err();
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

/// `get env-var` when the outer key exists but the inner key is
/// missing returns `EnvVarNotFound`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn get_env_var_outer_exists_inner_missing() {
    let mut settings = Settings::default();
    let mut inner = BTreeMap::new();
    inner.insert("HUB_IP".to_owned(), "192.168.1.1".to_owned());
    settings.env_vars.insert("shades".to_owned(), inner);
    let (_dir, path) = write_settings(&settings);

    let target = GetTarget::EnvVar {
        cmd_or_alias: "shades".to_owned(),
        var_name: "WRONG_KEY".to_owned(),
    };
    let err = run_get_from(target, quiet_ctx(), &path).unwrap_err();
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

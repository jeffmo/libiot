//! Tests for the `set` command handler.
//!
//! Written by Claude Code, reviewed by a human.

use crate::commands::set::set_alias_from;
use crate::commands::set::set_env_var_from;
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

/// `set alias` with a name that shadows a built-in returns
/// `AliasShadowsBuiltin`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_alias_shadows_builtin_returns_error() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let err = set_alias_from("some-cmd", "list", false, quiet_ctx(), &path).unwrap_err();
    assert!(
        matches!(
            err,
            CliError::AliasShadowsBuiltin { ref alias, ref builtin }
                if alias == "list" && builtin == "list"
        ),
        "expected AliasShadowsBuiltin, got {err:?}",
    );
}

/// `set alias` with `--overwrite` replaces an existing alias.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_alias_overwrite_replaces_existing() {
    let mut settings = Settings::default();
    settings
        .aliases
        .insert("shades".to_owned(), "old-cmd".to_owned());
    let (_dir, path) = write_settings(&settings);

    let result = set_alias_from("new-cmd", "shades", true, quiet_ctx(), &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");

    let reloaded = load_settings_from(&path).expect("reload settings");
    assert_eq!(
        reloaded.aliases.get("shades").map(String::as_str),
        Some("new-cmd")
    );
}

/// `set alias` without `--overwrite` on an existing alias returns
/// `AliasAlreadyExists`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_alias_no_overwrite_returns_already_exists() {
    let mut settings = Settings::default();
    settings
        .aliases
        .insert("shades".to_owned(), "old-cmd".to_owned());
    let (_dir, path) = write_settings(&settings);

    let err = set_alias_from("new-cmd", "shades", false, quiet_ctx(), &path).unwrap_err();
    assert!(
        matches!(
            err,
            CliError::AliasAlreadyExists { ref alias, ref target }
                if alias == "shades" && target == "old-cmd"
        ),
        "expected AliasAlreadyExists, got {err:?}",
    );
}

/// `set alias` on a fresh settings file creates the alias.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_alias_creates_new_alias() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let result = set_alias_from("some-cmd", "shades", false, quiet_ctx(), &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");

    let reloaded = load_settings_from(&path).expect("reload settings");
    assert_eq!(
        reloaded.aliases.get("shades").map(String::as_str),
        Some("some-cmd"),
    );
}

/// `set env-var` with a `LIBIOT_` prefix returns
/// `EnvVarLibiotPrefix`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_env_var_libiot_prefix_uppercase_returns_error() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let err = set_env_var_from("some-cmd", "LIBIOT_FOO", "bar", quiet_ctx(), &path).unwrap_err();
    assert!(
        matches!(err, CliError::EnvVarLibiotPrefix { ref name } if name == "LIBIOT_FOO"),
        "expected EnvVarLibiotPrefix, got {err:?}",
    );
}

/// `set env-var` with a lowercase `libiot_` prefix also returns
/// `EnvVarLibiotPrefix`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_env_var_libiot_prefix_lowercase_returns_error() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let err = set_env_var_from("some-cmd", "libiot_foo", "bar", quiet_ctx(), &path).unwrap_err();
    assert!(
        matches!(err, CliError::EnvVarLibiotPrefix { ref name } if name == "libiot_foo"),
        "expected EnvVarLibiotPrefix, got {err:?}",
    );
}

/// `set env-var` stores the variable under the literal
/// command/alias name.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_env_var_stores_under_literal_name() {
    let mut settings = Settings::default();
    settings
        .aliases
        .insert("shades".to_owned(), "rollease".to_owned());
    let (_dir, path) = write_settings(&settings);

    let result = set_env_var_from("shades", "HUB_IP", "192.168.1.1", quiet_ctx(), &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");

    let reloaded = load_settings_from(&path).expect("reload settings");
    let value = reloaded
        .env_vars
        .get("shades")
        .and_then(|m| m.get("HUB_IP"))
        .map(String::as_str);
    assert_eq!(value, Some("192.168.1.1"));
}

/// `set env-var` with mixed-case `LiBiOt_` prefix returns
/// `EnvVarLibiotPrefix`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_env_var_libiot_prefix_mixed_case_returns_error() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let err = set_env_var_from("some-cmd", "LiBiOt_Bar", "baz", quiet_ctx(), &path).unwrap_err();
    assert!(
        matches!(err, CliError::EnvVarLibiotPrefix { ref name } if name == "LiBiOt_Bar"),
        "expected EnvVarLibiotPrefix, got {err:?}",
    );
}

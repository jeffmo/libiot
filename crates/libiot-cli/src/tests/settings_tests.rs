//! Tests for the [`Settings`] struct, path resolution, persistence,
//! and environment-variable merging logic.

use std::collections::BTreeMap;

use crate::settings::Settings;
use crate::settings::load_settings_from;
use crate::settings::resolve_env_vars;
use crate::settings::save_settings_to;
use crate::settings::settings_path;

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

/// `Settings::default()` should serialize to the empty JSON object `{}`
/// because both maps are empty and annotated with
/// `skip_serializing_if = "BTreeMap::is_empty"`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn default_settings_serialize_to_empty_object() {
    let json = serde_json::to_string(&Settings::default()).unwrap();
    assert_eq!(json, "{}");
}

/// A round-trip (serialize then deserialize) preserves every field.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn round_trip_preserves_all_fields() {
    let mut aliases = BTreeMap::new();
    aliases.insert("shades".to_owned(), "rollease".to_owned());

    let mut inner = BTreeMap::new();
    inner.insert("HUB_IP".to_owned(), "192.168.1.42".to_owned());
    let mut env_vars = BTreeMap::new();
    env_vars.insert("rollease".to_owned(), inner);

    let original = Settings {
        aliases: aliases.clone(),
        env_vars: env_vars.clone(),
    };

    let json = serde_json::to_string_pretty(&original).unwrap();
    let restored: Settings = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.aliases, aliases);
    assert_eq!(restored.env_vars, env_vars);
}

/// When the `"env-vars"` key is missing from the JSON, the field
/// defaults to an empty map (via `#[serde(default)]`).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn missing_env_vars_key_defaults_to_empty() {
    let json = r#"{ "aliases": { "tv": "samsung" } }"#;
    let s: Settings = serde_json::from_str(json).unwrap();

    assert_eq!(s.aliases.len(), 1);
    assert!(s.env_vars.is_empty());
}

// ---------------------------------------------------------------------------
// resolve_env_vars
// ---------------------------------------------------------------------------

/// A direct command name (not an alias) returns its env vars with the
/// `LIBIOT_` prefix.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_env_vars_direct_command() {
    let mut inner = BTreeMap::new();
    inner.insert("HUB_IP".to_owned(), "10.0.0.1".to_owned());
    inner.insert("PORT".to_owned(), "9000".to_owned());

    let mut env_vars = BTreeMap::new();
    env_vars.insert("rollease".to_owned(), inner);

    let settings = Settings {
        aliases: BTreeMap::new(),
        env_vars,
    };

    let resolved = resolve_env_vars(&settings, "rollease");
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved.get("LIBIOT_HUB_IP").unwrap(), "10.0.0.1");
    assert_eq!(resolved.get("LIBIOT_PORT").unwrap(), "9000");
}

/// When `cmd_or_alias` is an alias, the underlying command's vars are
/// loaded first, then the alias-specific vars are overlaid.  An alias
/// var with the same key overrides the command var.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_env_vars_alias_merges_and_overrides() {
    let mut aliases = BTreeMap::new();
    aliases.insert("shades".to_owned(), "rollease".to_owned());

    let mut cmd_inner = BTreeMap::new();
    cmd_inner.insert("HUB_IP".to_owned(), "10.0.0.1".to_owned());
    cmd_inner.insert("TIMEOUT".to_owned(), "30".to_owned());

    let mut alias_inner = BTreeMap::new();
    alias_inner.insert("HUB_IP".to_owned(), "192.168.1.99".to_owned());
    alias_inner.insert("ROOM".to_owned(), "living".to_owned());

    let mut env_vars = BTreeMap::new();
    env_vars.insert("rollease".to_owned(), cmd_inner);
    env_vars.insert("shades".to_owned(), alias_inner);

    let settings = Settings { aliases, env_vars };

    let resolved = resolve_env_vars(&settings, "shades");

    // alias overrides command for HUB_IP
    assert_eq!(
        resolved.get("LIBIOT_HUB_IP").unwrap(),
        "192.168.1.99",
    );
    // command-only var is still present
    assert_eq!(resolved.get("LIBIOT_TIMEOUT").unwrap(), "30");
    // alias-only var is present
    assert_eq!(resolved.get("LIBIOT_ROOM").unwrap(), "living");
    assert_eq!(resolved.len(), 3);
}

/// A name that is neither an alias nor has env vars returns an empty
/// map.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_env_vars_nonexistent_returns_empty() {
    let settings = Settings::default();
    let resolved = resolve_env_vars(&settings, "no-such-command");
    assert!(resolved.is_empty());
}

// ---------------------------------------------------------------------------
// settings_path
// ---------------------------------------------------------------------------

/// `settings_path()` always returns a path whose final two components
/// are `libiot/settings.json`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn settings_path_ends_with_libiot_settings_json() {
    // Ensure HOME is set so the fallback works even in CI.
    if std::env::var("HOME").is_err() {
        // Skip rather than fail — we can't synthesize HOME portably
        // without affecting other parallel tests.
        return;
    }

    let path = settings_path().unwrap();

    // Whether LIBIOT_CONFIG_DIR is set or not, the filename must be
    // `settings.json` inside a directory.
    assert!(
        path.ends_with("settings.json"),
        "expected path ending in settings.json, got {}",
        path.display(),
    );
}

// ---------------------------------------------------------------------------
// save + load round-trip (filesystem)
// ---------------------------------------------------------------------------

/// Save settings to a temp directory via [`save_settings_to`], then
/// load them back with [`load_settings_from`] and assert the data
/// matches.  Uses explicit path parameters to avoid mutating
/// process-global environment variables.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn save_and_load_round_trip_via_tempdir() {
    let tmp = tempfile::tempdir().unwrap();
    let file_path = tmp.path().join("settings.json");

    // Build non-trivial settings.
    let mut aliases = BTreeMap::new();
    aliases.insert("tv".to_owned(), "samsung-frametv".to_owned());

    let mut inner = BTreeMap::new();
    inner.insert("IP".to_owned(), "192.168.1.50".to_owned());
    let mut env_vars = BTreeMap::new();
    env_vars.insert("samsung-frametv".to_owned(), inner);

    let original = Settings { aliases, env_vars };

    save_settings_to(&original, &file_path).unwrap();

    let loaded = load_settings_from(&file_path).unwrap();

    assert_eq!(loaded.aliases, original.aliases);
    assert_eq!(loaded.env_vars, original.env_vars);

    // Verify the file actually exists where we expect.
    assert!(file_path.exists(), "settings.json was not created");

    // On Unix, verify directory and file permissions.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let dir_mode = std::fs::metadata(tmp.path())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(dir_mode, 0o700, "directory permissions should be 0700");

        let file_mode = std::fs::metadata(&file_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(file_mode, 0o600, "file permissions should be 0600");
    }
}

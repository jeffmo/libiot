//! Settings persistence — load, save, and query the `settings.json`
//! file that stores aliases and per-command environment variables.
//!
//! The canonical location is `~/.config/libiot/settings.json`, which can
//! be overridden by setting the `LIBIOT_CONFIG_DIR` environment
//! variable.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::error::CliError;
use crate::error::CliResult;

// ---------------------------------------------------------------------------
// Settings struct
// ---------------------------------------------------------------------------

/// Persistent CLI settings: alias mappings and per-command environment
/// variables.
///
/// Serialized as pretty JSON.  [`BTreeMap`] is used instead of
/// `HashMap` so that keys appear in deterministic (sorted) order.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Settings {
    /// Command aliases (`alias-name` -> `target-command`).
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub aliases: BTreeMap<String, String>,

    /// Per-command environment variables.
    ///
    /// Outer key = command-or-alias name, inner map = var name -> value.
    /// Variable names are stored *without* the `LIBIOT_` prefix; the
    /// prefix is prepended at resolution time by [`resolve_env_vars`].
    #[serde(default)]
    #[serde(rename = "env-vars")]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub env_vars: BTreeMap<String, BTreeMap<String, String>>,
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Return the libiot configuration directory.
///
/// Checks `LIBIOT_CONFIG_DIR` first; falls back to
/// `$HOME/.config/libiot`.
pub(crate) fn config_dir() -> CliResult<PathBuf> {
    match std::env::var("LIBIOT_CONFIG_DIR") {
        Ok(d) if !d.is_empty() => Ok(PathBuf::from(d)),
        _ => {
            let home = std::env::var("HOME").map_err(|_| CliError::NoHomeDir)?;
            if home.is_empty() {
                return Err(CliError::NoHomeDir);
            }
            Ok(PathBuf::from(home).join(".config").join("libiot"))
        },
    }
}

/// Return the canonical settings file path.
///
/// Checks `LIBIOT_CONFIG_DIR` first; falls back to
/// `$HOME/.config/libiot/settings.json`.
pub(crate) fn settings_path() -> CliResult<PathBuf> {
    Ok(config_dir()?.join("settings.json"))
}

// ---------------------------------------------------------------------------
// Load / save
// ---------------------------------------------------------------------------

/// Load settings from disk.
///
/// Returns [`Settings::default()`] when the file does not exist.
/// Returns [`CliError::SettingsReadError`] if the file exists but
/// cannot be read, and [`CliError::SettingsParseError`] if it contains
/// invalid JSON.
pub(crate) fn load_settings() -> CliResult<Settings> {
    load_settings_from(&settings_path()?)
}

/// Load settings from a specific file path.
///
/// Same semantics as [`load_settings`] but reads from the given path
/// instead of the canonical location. Used by tests to avoid mutating
/// process-global environment variables.
pub(crate) fn load_settings_from(path: &Path) -> CliResult<Settings> {
    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Settings::default());
        },
        Err(e) => {
            return Err(CliError::SettingsReadError {
                path: path.display().to_string(),
                source: e,
            });
        },
    };

    serde_json::from_str(&contents).map_err(|e| CliError::SettingsParseError {
        path: path.display().to_string(),
        source: e,
    })
}

/// Write settings to disk atomically.
///
/// 1. Creates the parent directory (mode 0700 on Unix) if needed.
/// 2. Writes to a `.tmp` sibling file.
/// 3. Sets mode 0600 on the temp file (Unix).
/// 4. Renames the temp file over the real file (atomic on the same FS).
pub(crate) fn save_settings(settings: &Settings) -> CliResult<()> {
    save_settings_to(settings, &settings_path()?)
}

/// Write settings to a specific file path atomically.
///
/// Same semantics as [`save_settings`] but writes to the given path
/// instead of the canonical location. Used by tests to avoid mutating
/// process-global environment variables.
pub(crate) fn save_settings_to(settings: &Settings, path: &Path) -> CliResult<()> {
    // -- ensure directory exists ------------------------------------------
    let dir = path
        .parent()
        // `settings_path()` always returns a file inside a directory, so
        // `parent()` will not be `None`.  Guard defensively anyway.
        .ok_or_else(|| CliError::SettingsDirError {
            path: path.display().to_string(),
            source: std::io::Error::other("settings path has no parent directory"),
        })?;

    fs::create_dir_all(dir).map_err(|e| CliError::SettingsDirError {
        path: dir.display().to_string(),
        source: e,
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).map_err(|e| {
            CliError::SettingsPermissionError {
                path: dir.display().to_string(),
                source: e,
            }
        })?;
    }

    // -- serialize --------------------------------------------------------
    let json =
        serde_json::to_string_pretty(settings).map_err(|e| CliError::SettingsWriteError {
            path: path.display().to_string(),
            source: std::io::Error::other(e),
        })?;

    // -- write to temp file -----------------------------------------------
    let tmp_path = path.with_extension("json.tmp");

    fs::write(&tmp_path, json.as_bytes()).map_err(|e| CliError::SettingsWriteError {
        path: tmp_path.display().to_string(),
        source: e,
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600)).map_err(|e| {
            CliError::SettingsPermissionError {
                path: tmp_path.display().to_string(),
                source: e,
            }
        })?;
    }

    // -- atomic rename ----------------------------------------------------
    fs::rename(&tmp_path, path).map_err(|e| CliError::SettingsWriteError {
        path: path.display().to_string(),
        source: e,
    })?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Env-var resolution
// ---------------------------------------------------------------------------

/// Resolve effective environment variables for a command or alias.
///
/// The returned map has keys prefixed with `LIBIOT_`.
///
/// Resolution order:
/// 1. If `cmd_or_alias` is a known alias, look up the underlying
///    command name; otherwise treat `cmd_or_alias` as the command.
/// 2. Collect env vars for the underlying command (if any).
/// 3. If `cmd_or_alias` was an alias, overlay its alias-specific vars
///    (alias vars override command vars for the same key).
/// 4. Prepend `LIBIOT_` to every key.
pub(crate) fn resolve_env_vars(
    settings: &Settings,
    cmd_or_alias: &str,
) -> BTreeMap<String, String> {
    let mut merged: BTreeMap<String, String> = BTreeMap::new();

    if let Some(target_cmd) = settings.aliases.get(cmd_or_alias) {
        // `cmd_or_alias` is an alias — start with the target command's
        // vars, then overlay alias-specific vars.
        if let Some(cmd_vars) = settings.env_vars.get(target_cmd.as_str()) {
            for (k, v) in cmd_vars {
                merged.insert(k.clone(), v.clone());
            }
        }
        if let Some(alias_vars) = settings.env_vars.get(cmd_or_alias) {
            for (k, v) in alias_vars {
                merged.insert(k.clone(), v.clone());
            }
        }
    } else {
        // Not an alias — direct command lookup.
        if let Some(cmd_vars) = settings.env_vars.get(cmd_or_alias) {
            for (k, v) in cmd_vars {
                merged.insert(k.clone(), v.clone());
            }
        }
    }

    // Prefix every key with LIBIOT_.
    merged
        .into_iter()
        .map(|(k, v)| (format!("LIBIOT_{k}"), v))
        .collect()
}

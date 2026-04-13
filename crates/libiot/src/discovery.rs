//! PATH scanning for installed `libiot-*` CLI binaries.
//!
//! [`discover_clis`] walks every directory on `$PATH` looking for
//! executables whose name starts with `libiot-`. The first match for a
//! given short name wins (matching Unix shell resolution order).
//! [`find_cli`] does the same but short-circuits on a single target.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

/// A CLI binary discovered on `$PATH`.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct DiscoveredCli {
    /// Short name (e.g. "rollease-automate-pulse-pro-hub").
    pub name: String,
    /// Full path to the binary.
    pub path: PathBuf,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Scan `$PATH` for all binaries matching `libiot-*`.
///
/// Returns them sorted by name. Deduplicates: the first `$PATH` match
/// with the executable bit set wins (matching Unix shell resolution).
pub(crate) fn discover_clis() -> Vec<DiscoveredCli> {
    let Ok(path_str) = std::env::var("PATH") else {
        return Vec::new();
    };
    discover_clis_with_path(&path_str)
}

/// Check if a specific `libiot-{name}` binary exists on `$PATH`.
///
/// Returns `Some(path)` if found, `None` if not.
pub(crate) fn find_cli(name: &str) -> Option<PathBuf> {
    let Ok(path_str) = std::env::var("PATH") else {
        return None;
    };
    find_cli_with_path(name, &path_str)
}

// ---------------------------------------------------------------------------
// Internal helpers (testable without mutating process env)
// ---------------------------------------------------------------------------

/// Scan a given PATH string for all `libiot-*` executables.
///
/// Same semantics as [`discover_clis`] but operates on the supplied
/// `path_str` rather than reading `$PATH`.
pub(crate) fn discover_clis_with_path(path_str: &str) -> Vec<DiscoveredCli> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut results: BTreeMap<String, DiscoveredCli> = BTreeMap::new();

    for dir in std::env::split_paths(path_str) {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let Ok(file_name) = entry.file_name().into_string() else {
                continue;
            };

            let Some(short_name) = file_name.strip_prefix("libiot-") else {
                continue;
            };

            if seen.contains(short_name) {
                continue;
            }

            if !is_executable(&entry) {
                continue;
            }

            let short_name = short_name.to_owned();
            seen.insert(short_name.clone());
            results.insert(
                short_name.clone(),
                DiscoveredCli {
                    name: short_name,
                    path: entry.path(),
                },
            );
        }
    }

    results.into_values().collect()
}

/// Search a given PATH string for a specific `libiot-{name}` binary.
///
/// Same semantics as [`find_cli`] but operates on the supplied
/// `path_str` rather than reading `$PATH`.
pub(crate) fn find_cli_with_path(name: &str, path_str: &str) -> Option<PathBuf> {
    let target = format!("libiot-{name}");

    for dir in std::env::split_paths(path_str) {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let Ok(file_name) = entry.file_name().into_string() else {
                continue;
            };

            if file_name != target {
                continue;
            }

            if !is_executable(&entry) {
                continue;
            }

            return Some(entry.path());
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Platform-specific executable check
// ---------------------------------------------------------------------------

/// Returns `true` if the directory entry looks like an executable file.
///
/// On Unix: checks `is_file()` and the executable bit.
/// On non-Unix: checks `is_file()` only.
fn is_executable(entry: &fs::DirEntry) -> bool {
    let Ok(metadata) = entry.metadata() else {
        return false;
    };

    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

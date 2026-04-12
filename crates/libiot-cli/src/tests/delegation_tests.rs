//! Tests for the delegation resolution logic in [`crate::delegation`].

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::delegation::resolve_delegation_with;
use crate::error::CliError;
use crate::settings::Settings;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a file inside `dir` with the given name. On Unix, optionally
/// set it executable.
fn create_file(dir: &Path, name: &str, executable: bool) {
    let path = dir.join(name);
    fs::write(&path, "#!/bin/sh\n").unwrap();
    if executable {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        }
    }
}

/// Build a PATH string from a slice of directory paths, joined with the
/// platform separator.
fn make_path(dirs: &[&Path]) -> String {
    std::env::join_paths(dirs).unwrap().into_string().unwrap()
}

/// Construct a [`Settings`] with the given aliases and env vars.
fn make_settings(
    aliases: &[(&str, &str)],
    env_vars: &[(&str, &[(&str, &str)])],
) -> Settings {
    let mut s = Settings::default();
    for (alias, target) in aliases {
        s.aliases.insert((*alias).to_owned(), (*target).to_owned());
    }
    for (cmd, vars) in env_vars {
        let mut map = BTreeMap::new();
        for (k, v) in *vars {
            map.insert((*k).to_owned(), (*v).to_owned());
        }
        s.env_vars.insert((*cmd).to_owned(), map);
    }
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// A direct command name (not an alias) resolves to the corresponding
/// `libiot-{name}` binary on PATH.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn direct_command_resolves_to_binary() {
    let tmp = tempfile::tempdir().unwrap();
    create_file(tmp.path(), "libiot-gadget", /* executable = */ true);

    let path_str = make_path(&[tmp.path()]);
    let settings = Settings::default();

    let (binary, env_vars) = resolve_delegation_with("gadget", &settings, &path_str).unwrap();

    assert_eq!(binary, tmp.path().join("libiot-gadget"));
    assert!(env_vars.is_empty(), "no env vars configured");
}

/// When `name` is a known alias, the resolution looks up the alias
/// target on PATH and returns the binary for that target.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn alias_resolves_to_target_binary() {
    let tmp = tempfile::tempdir().unwrap();
    create_file(tmp.path(), "libiot-real-device", /* executable = */ true);

    let path_str = make_path(&[tmp.path()]);
    let settings = make_settings(&[("shortcut", "real-device")], &[]);

    let (binary, env_vars) =
        resolve_delegation_with("shortcut", &settings, &path_str).unwrap();

    assert_eq!(binary, tmp.path().join("libiot-real-device"));
    assert!(env_vars.is_empty(), "no env vars configured");
}

/// When an alias has env vars configured on both the alias and its
/// underlying target command, the resolved env vars contain the
/// merged set with LIBIOT_ prefix — and alias vars override target
/// vars for the same key.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn alias_with_env_vars_merges_and_prefixes() {
    let tmp = tempfile::tempdir().unwrap();
    create_file(tmp.path(), "libiot-hub", /* executable = */ true);

    let path_str = make_path(&[tmp.path()]);
    let settings = make_settings(
        &[("myhub", "hub")],
        &[
            ("hub", &[("HOST", "192.168.1.1"), ("PORT", "8080")]),
            ("myhub", &[("HOST", "10.0.0.1"), ("ZONE", "living-room")]),
        ],
    );

    let (_binary, env_vars) =
        resolve_delegation_with("myhub", &settings, &path_str).unwrap();

    // Alias HOST overrides command HOST.
    assert_eq!(env_vars.get("LIBIOT_HOST").unwrap(), "10.0.0.1");
    // Command PORT inherited (alias doesn't override).
    assert_eq!(env_vars.get("LIBIOT_PORT").unwrap(), "8080");
    // Alias-only ZONE is present.
    assert_eq!(env_vars.get("LIBIOT_ZONE").unwrap(), "living-room");
    assert_eq!(env_vars.len(), 3);
}

/// A name that is neither a known alias nor a `libiot-*` binary on
/// PATH produces a `DelegationTargetNotFound` error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unknown_name_returns_delegation_target_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    // Put an unrelated binary on PATH so the directory isn't empty.
    create_file(tmp.path(), "libiot-other", /* executable = */ true);

    let path_str = make_path(&[tmp.path()]);
    let settings = Settings::default();

    let err = resolve_delegation_with("nonexistent", &settings, &path_str).unwrap_err();

    match err {
        CliError::DelegationTargetNotFound { name } => {
            assert_eq!(name, "nonexistent");
        },
        other => panic!("expected DelegationTargetNotFound, got {other:?}"),
    }
}

/// An alias whose target command has no corresponding `libiot-*` binary
/// on PATH produces a `DelegationTargetNotFound` error (using the
/// original alias name, not the target).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn alias_target_not_on_path_returns_delegation_target_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    // The alias points to "missing-device", but no libiot-missing-device exists.
    create_file(tmp.path(), "libiot-other", /* executable = */ true);

    let path_str = make_path(&[tmp.path()]);
    let settings = make_settings(&[("shortcut", "missing-device")], &[]);

    let err = resolve_delegation_with("shortcut", &settings, &path_str).unwrap_err();

    match err {
        CliError::DelegationTargetNotFound { name } => {
            assert_eq!(name, "shortcut", "error should report the alias name");
        },
        other => panic!("expected DelegationTargetNotFound, got {other:?}"),
    }
}

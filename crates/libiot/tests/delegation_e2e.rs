//! End-to-end tests for delegation to discovered `libiot-*` CLIs.
//!
//! Each test creates a fake `libiot-test-device` shell script in a
//! temporary directory, adds that directory to `PATH` via
//! `Command::env`, and exercises delegation through the compiled
//! `libiot` binary.

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;

use predicates::str::contains;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temporary directory containing a fake `libiot-test-device`
/// script that echoes its arguments and `LIBIOT_`-prefixed environment
/// variables. Returns `(tempdir_handle, path_string)`.
fn fake_cli_dir() -> (TempDir, String) {
    let dir = TempDir::new().expect("create tempdir for fake CLI");
    let script_path = dir.path().join("libiot-test-device");
    fs::write(
        &script_path,
        "#!/bin/sh\necho \"ARGS:$*\"\n/usr/bin/env | /usr/bin/grep LIBIOT_ | /usr/bin/sort\n",
    )
    .expect("write fake CLI");
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755)).expect("chmod fake CLI");
    let path_str = dir.path().display().to_string();
    (dir, path_str)
}

/// Build a PATH string that prepends the fake CLI directory to the
/// current system PATH, so both discovery and the exec'd script can
/// find what they need.
fn path_with_fake(fake_path: &str) -> String {
    format!(
        "{}:{}",
        fake_path,
        std::env::var("PATH").unwrap_or_default()
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Delegating to a fake CLI should exec into the script and pass
/// through the remaining arguments.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn delegation_to_fake_cli() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    let mut cmd = common::libiot_cmd(config.path());
    cmd.env("PATH", path_with_fake(&fake_path));
    cmd.args(["test-device", "arg1", "arg2"])
        .assert()
        .success()
        .stdout(contains("ARGS:arg1 arg2"));
}

/// Delegating to a command with configured env vars should make those
/// variables visible to the child script (prefixed with `LIBIOT_`).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn delegation_with_env_vars() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    let full_path = path_with_fake(&fake_path);

    // Set an env var for the command.
    let mut set_cmd = common::libiot_cmd(config.path());
    set_cmd.env("PATH", &full_path);
    set_cmd
        .args(["set", "env-var", "test-device", "HUB_IP", "192.168.1.42"])
        .assert()
        .success();

    // Delegate — the script should see LIBIOT_HUB_IP.
    let mut delegate_cmd = common::libiot_cmd(config.path());
    delegate_cmd.env("PATH", &full_path);
    delegate_cmd
        .args(["test-device", "hello"])
        .assert()
        .success()
        .stdout(contains("LIBIOT_HUB_IP=192.168.1.42"));
}

/// Delegating to a command that does not exist on `PATH` should fail
/// with exit code 17 (`DelegationTargetNotFound`).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn delegation_unknown_command() {
    let config = TempDir::new().expect("config tempdir");

    // Use an empty PATH so nothing is discoverable.
    let empty_dir = TempDir::new().expect("empty tempdir");

    let mut cmd = common::libiot_cmd(config.path());
    cmd.env("PATH", empty_dir.path());
    cmd.args(["nonexistent-device", "arg1"]).assert().code(17);
}

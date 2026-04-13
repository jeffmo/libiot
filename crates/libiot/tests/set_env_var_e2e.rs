//! End-to-end tests for environment variable set / get / unset
//! round-trips.
//!
//! Each test launches the compiled `libiot` binary via `assert_cmd`,
//! pointing `LIBIOT_CONFIG_DIR` at a temporary directory and placing a
//! fake `libiot-test-device` script on `PATH` so that target validation
//! succeeds.

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;

use predicates::str::contains;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temporary directory containing a fake `libiot-test-device`
/// executable. Returns `(tempdir_handle, path_string)`.
fn fake_cli_dir() -> (TempDir, String) {
    let dir = TempDir::new().expect("create tempdir for fake CLI");
    let script_path = dir.path().join("libiot-test-device");
    fs::write(&script_path, "#!/bin/sh\ntrue\n").expect("write fake CLI");
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755)).expect("chmod fake CLI");
    let path_str = dir.path().display().to_string();
    (dir, path_str)
}

/// Build a command with `LIBIOT_CONFIG_DIR` and `PATH` configured.
fn cmd_with_fake_cli(config_dir: &std::path::Path, fake_path: &str) -> assert_cmd::Command {
    let mut cmd = common::libiot_cmd(config_dir);
    cmd.env(
        "PATH",
        format!(
            "{}:{}",
            fake_path,
            std::env::var("PATH").unwrap_or_default()
        ),
    );
    cmd
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Set an environment variable for a PATH-discoverable command and then
/// get it back; the output should contain the original value.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_and_get_env_var_round_trip() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    // Set the env var.
    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "env-var", "test-device", "HUB_IP", "192.168.1.42"])
        .assert()
        .success();

    // Get the env var — human output prints only the value.
    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["get", "env-var", "test-device", "HUB_IP"])
        .assert()
        .success()
        .stdout(contains("192.168.1.42"));
}

/// Setting an env var whose name starts with `LIBIOT_` (case-insensitive)
/// should fail with exit code 18 (`EnvVarLibiotPrefix`).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_env_var_with_libiot_prefix() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "env-var", "test-device", "LIBIOT_SOMETHING", "bad"])
        .assert()
        .code(18);
}

/// Unsetting an env var should remove it; a subsequent `get` should fail
/// with exit code 19 (`EnvVarNotFound`).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unset_env_var() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    // Set the env var first.
    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "env-var", "test-device", "HUB_IP", "192.168.1.42"])
        .assert()
        .success();

    // Unset it.
    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["unset", "env-var", "test-device", "HUB_IP"])
        .assert()
        .success();

    // Get should now fail.
    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["get", "env-var", "test-device", "HUB_IP"])
        .assert()
        .code(19);
}

/// Getting an env var with `--format json` should produce valid JSON
/// containing the variable name and value.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn get_env_var_json_output() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    // Set the env var first.
    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "env-var", "test-device", "HUB_IP", "192.168.1.42"])
        .assert()
        .success();

    // Get with JSON output.
    let output = cmd_with_fake_cli(config.path(), &fake_path)
        .args([
            "--format",
            "json",
            "get",
            "env-var",
            "test-device",
            "HUB_IP",
        ])
        .output()
        .expect("run command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("valid UTF-8");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(parsed["name"], "HUB_IP");
    assert_eq!(parsed["value"], "192.168.1.42");
}

//! End-to-end tests for alias set / get / unset round-trips.
//!
//! Each test launches the compiled `libiot` binary via `assert_cmd`,
//! pointing `LIBIOT_CONFIG_DIR` at a temporary directory and placing a
//! fake `libiot-test-device` script on `PATH` so that alias-target
//! validation succeeds.

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;

use predicates::str::contains;
use predicates::str::is_empty;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temporary directory containing a fake `libiot-test-device`
/// executable. Returns `(tempdir_handle, path_string)` — the tempdir
/// handle must be kept alive for the duration of the test.
fn fake_cli_dir() -> (TempDir, String) {
    let dir = TempDir::new().expect("create tempdir for fake CLI");
    let script_path = dir.path().join("libiot-test-device");
    fs::write(&script_path, "#!/bin/sh\ntrue\n").expect("write fake CLI");
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755)).expect("chmod fake CLI");
    let path_str = dir.path().display().to_string();
    (dir, path_str)
}

/// Build a command that has both `LIBIOT_CONFIG_DIR` and `PATH` set so
/// that alias-target validation can find `libiot-test-device`.
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

/// Set an alias and then get it back; the output should match the
/// original target command name.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_and_get_alias_round_trip() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    // Set the alias.
    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "alias", "test-device", "td"])
        .assert()
        .success();

    // Get the alias — human output prints only the target.
    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["get", "alias", "td"])
        .assert()
        .success()
        .stdout(contains("test-device"));
}

/// Setting the same alias twice without `--overwrite` should fail with
/// exit code 10 (`AliasAlreadyExists`).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_alias_already_exists() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "alias", "test-device", "td"])
        .assert()
        .success();

    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "alias", "test-device", "td"])
        .assert()
        .code(10);
}

/// Setting the same alias twice with `-f` (overwrite) should succeed.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_alias_with_overwrite() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "alias", "test-device", "td"])
        .assert()
        .success();

    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "alias", "test-device", "td", "-f"])
        .assert()
        .success();
}

/// Unsetting an alias should remove it; a subsequent `get` should fail
/// with exit code 11 (`AliasNotFound`).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unset_alias() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "alias", "test-device", "td"])
        .assert()
        .success();

    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["unset", "alias", "td"])
        .assert()
        .success();

    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["get", "alias", "td"])
        .assert()
        .code(11);
}

/// Attempting to set an alias that shadows a built-in command should
/// fail with exit code 12 (`AliasShadowsBuiltin`).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_alias_shadows_builtin() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "alias", "test-device", "install"])
        .assert()
        .code(12);
}

/// Setting an alias with `--format json` should produce a JSON response
/// containing `"ok": true`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_alias_json_output() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    let output = cmd_with_fake_cli(config.path(), &fake_path)
        .args(["--format", "json", "set", "alias", "test-device", "td"])
        .output()
        .expect("run command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("valid UTF-8");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(parsed["ok"], serde_json::Value::Bool(true));
    assert!(parsed["message"].as_str().unwrap_or("").contains("td"));
}

/// Setting an alias with `-q` (quiet) should produce no stdout output.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_alias_quiet_mode() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["-q", "set", "alias", "test-device", "td"])
        .assert()
        .success()
        .stdout(is_empty());
}

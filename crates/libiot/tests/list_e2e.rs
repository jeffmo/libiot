//! End-to-end tests for the `list` and `config-path` built-in
//! commands.
//!
//! Each test launches the compiled `libiot` binary via `assert_cmd`,
//! pointing `LIBIOT_CONFIG_DIR` at a temporary directory.

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

/// Listing aliases when none are configured should succeed and output
/// the "(no aliases configured)" sentinel.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_aliases_empty() {
    let config = TempDir::new().expect("config tempdir");

    common::libiot_cmd(config.path())
        .args(["list", "aliases"])
        .assert()
        .success()
        .stdout(contains("no aliases configured"));
}

/// After setting some aliases, `list aliases` should include them in
/// the output.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_aliases_with_data() {
    let config = TempDir::new().expect("config tempdir");
    let (_fake, fake_path) = fake_cli_dir();

    // Create two aliases.
    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "alias", "test-device", "td"])
        .assert()
        .success();

    cmd_with_fake_cli(config.path(), &fake_path)
        .args(["set", "alias", "test-device", "my-device"])
        .assert()
        .success();

    // List them — human output has "alias -> target" lines.
    common::libiot_cmd(config.path())
        .args(["list", "aliases"])
        .assert()
        .success()
        .stdout(contains("td -> test-device"))
        .stdout(contains("my-device -> test-device"));
}

/// `libiot config-path` should output a path that ends with
/// `settings.json`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn config_path_outputs_settings_json() {
    let config = TempDir::new().expect("config tempdir");

    let output = common::libiot_cmd(config.path())
        .args(["config-path"])
        .output()
        .expect("run command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("valid UTF-8");
    let trimmed = stdout.trim();
    assert!(
        trimmed.ends_with("settings.json"),
        "config-path output should end with settings.json, got: {trimmed:?}"
    );
}

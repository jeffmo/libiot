//! Shared helpers for end-to-end integration tests.

use std::path::Path;

use assert_cmd::Command;

/// Build a [`Command`] for the `libiot` binary with `LIBIOT_CONFIG_DIR`
/// pointed at the given directory.
///
/// Written by Claude Code, reviewed by a human.
pub fn libiot_cmd(config_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("libiot").expect("binary exists");
    cmd.env("LIBIOT_CONFIG_DIR", config_dir);
    cmd
}

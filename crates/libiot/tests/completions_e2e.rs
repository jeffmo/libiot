//! End-to-end regression tests for shell completion generation.
//!
//! These tests verify that the generated completion scripts offer the
//! correct set of values for each positional argument — specifically
//! that CLI names, aliases, and "no completions" are assigned to the
//! right args.

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use assert_cmd::Command;

/// Create a fake `libiot-<name>` executable in `dir`.
fn create_fake_cli(dir: &Path, name: &str) {
    let path = dir.join(format!("libiot-{name}"));
    fs::write(
        &path,
        format!("#!/bin/sh\necho \"Control {name} device\"\n"),
    )
    .expect("write fake cli");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
}

/// Build a Command with both `LIBIOT_CONFIG_DIR` and `PATH` set.
fn libiot_with_path(config_dir: &Path, path_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("libiot").expect("binary exists");
    cmd.env("LIBIOT_CONFIG_DIR", config_dir);
    cmd.env("PATH", path_dir);
    cmd
}

/// Generate zsh completions and return the written file contents.
fn generate_zsh(config_dir: &Path, path_dir: &Path) -> String {
    libiot_with_path(config_dir, path_dir)
        .args(["completions", "zsh"])
        .assert()
        .success();
    let comp_file = config_dir.join("completions").join("zsh");
    fs::read_to_string(comp_file).expect("read completions file")
}

/// Set up an alias by running `libiot set alias`.
fn set_alias(config_dir: &Path, path_dir: &Path, cmd: &str, alias: &str) {
    libiot_with_path(config_dir, path_dir)
        .args(["set", "alias", cmd, alias])
        .assert()
        .success();
}

// -----------------------------------------------------------------------
// install: should NOT offer completions
// -----------------------------------------------------------------------

/// `libiot install <TAB>` should not suggest any CLI names or aliases.
/// The install arg uses `_default` (filesystem fallback), not a values
/// list, because we don't know which crates are available to install.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn install_name_has_no_completions() {
    let path_dir = tempfile::tempdir().expect("tempdir");
    let config_dir = tempfile::tempdir().expect("tempdir");
    create_fake_cli(path_dir.path(), "my-device");

    let script = generate_zsh(config_dir.path(), path_dir.path());

    // Find the install section's name arg — it should have :_default,
    // NOT :(my-device).
    for line in script.lines() {
        if line.contains(":name -- Crate to install") {
            assert!(
                line.contains(":_default"),
                "install name arg should use _default, got: {line}"
            );
            assert!(
                !line.contains("my-device"),
                "install name arg should NOT list CLI names, got: {line}"
            );
            return;
        }
    }
    panic!("did not find install name arg in completions script");
}

// -----------------------------------------------------------------------
// uninstall: should only offer CLI names, not aliases
// -----------------------------------------------------------------------

/// `libiot uninstall <TAB>` should suggest installed CLI names but NOT
/// aliases. Aliases are a libiot concept, not a cargo concept.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn uninstall_name_offers_clis_not_aliases() {
    let path_dir = tempfile::tempdir().expect("tempdir");
    let config_dir = tempfile::tempdir().expect("tempdir");
    create_fake_cli(path_dir.path(), "my-device");
    set_alias(config_dir.path(), path_dir.path(), "my-device", "mydev");

    let script = generate_zsh(config_dir.path(), path_dir.path());

    for line in script.lines() {
        if line.contains(":name -- Installed CLI to remove") {
            assert!(
                line.contains("my-device"),
                "uninstall should offer CLI name, got: {line}"
            );
            assert!(
                !line.contains("mydev"),
                "uninstall should NOT offer alias name, got: {line}"
            );
            return;
        }
    }
    panic!("did not find uninstall name arg in completions script");
}

// -----------------------------------------------------------------------
// update: should only offer CLI names, not aliases
// -----------------------------------------------------------------------

/// `libiot update <TAB>` should suggest installed CLI names but NOT
/// aliases. Update operates on cargo crate names, not aliases.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn update_name_offers_clis_not_aliases() {
    let path_dir = tempfile::tempdir().expect("tempdir");
    let config_dir = tempfile::tempdir().expect("tempdir");
    create_fake_cli(path_dir.path(), "my-device");
    set_alias(config_dir.path(), path_dir.path(), "my-device", "mydev");

    let script = generate_zsh(config_dir.path(), path_dir.path());

    for line in script.lines() {
        if line.contains(":name -- CLI to update") {
            assert!(
                line.contains("my-device"),
                "update should offer CLI name, got: {line}"
            );
            assert!(
                !line.contains("mydev"),
                "update should NOT offer alias name, got: {line}"
            );
            return;
        }
    }
    panic!("did not find update name arg in completions script");
}

// -----------------------------------------------------------------------
// set alias <CMD>: should only offer CLI names, not aliases
// -----------------------------------------------------------------------

/// `libiot set alias <CMD> <TAB>` should suggest installed CLI names
/// but NOT aliases (we don't support aliases-of-aliases).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_alias_cmd_offers_clis_not_aliases() {
    let path_dir = tempfile::tempdir().expect("tempdir");
    let config_dir = tempfile::tempdir().expect("tempdir");
    create_fake_cli(path_dir.path(), "my-device");
    set_alias(config_dir.path(), path_dir.path(), "my-device", "mydev");

    let script = generate_zsh(config_dir.path(), path_dir.path());

    for line in script.lines() {
        if line.contains(":cmd -- Target command name") {
            assert!(
                line.contains("my-device"),
                "set alias cmd should offer CLI name, got: {line}"
            );
            assert!(
                !line.contains("mydev"),
                "set alias cmd should NOT offer alias name, got: {line}"
            );
            return;
        }
    }
    panic!("did not find set alias cmd arg in completions script");
}

// -----------------------------------------------------------------------
// set alias: alias_name should NOT get value completions
// -----------------------------------------------------------------------

/// `libiot set alias CMD <ALIAS_NAME>` — the alias name is user-
/// invented, so it should use _default (not list existing aliases).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_alias_alias_name_has_no_completions() {
    let path_dir = tempfile::tempdir().expect("tempdir");
    let config_dir = tempfile::tempdir().expect("tempdir");
    create_fake_cli(path_dir.path(), "my-device");
    set_alias(config_dir.path(), path_dir.path(), "my-device", "mydev");

    let script = generate_zsh(config_dir.path(), path_dir.path());

    for line in script.lines() {
        if line.contains(":alias_name -- Alias name to create") {
            assert!(
                line.contains(":_default"),
                "set alias alias_name should use _default, got: {line}"
            );
            return;
        }
    }
    panic!("did not find set alias alias_name arg in completions script");
}

// -----------------------------------------------------------------------
// set env-var, get env-var, unset env-var: should offer CLIs + aliases
// -----------------------------------------------------------------------

/// `libiot set env-var <CMD_OR_ALIAS> <TAB>` should suggest both CLI
/// names and aliases.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn set_env_var_cmd_or_alias_offers_both() {
    let path_dir = tempfile::tempdir().expect("tempdir");
    let config_dir = tempfile::tempdir().expect("tempdir");
    create_fake_cli(path_dir.path(), "my-device");
    set_alias(config_dir.path(), path_dir.path(), "my-device", "mydev");

    let script = generate_zsh(config_dir.path(), path_dir.path());

    let mut found = false;
    for line in script.lines() {
        if line.contains(":cmd_or_alias -- Command or alias name") {
            assert!(
                line.contains("my-device"),
                "set env-var cmd_or_alias should offer CLI name, got: {line}"
            );
            assert!(
                line.contains("mydev"),
                "set env-var cmd_or_alias should offer alias name, got: {line}"
            );
            found = true;
        }
    }
    assert!(
        found,
        "did not find any cmd_or_alias arg in completions script"
    );
}

// -----------------------------------------------------------------------
// get alias, unset alias: should offer only aliases
// -----------------------------------------------------------------------

/// `libiot get alias <TAB>` and `libiot unset alias <TAB>` should
/// suggest configured aliases but NOT CLI names.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn get_unset_alias_offers_only_aliases() {
    let path_dir = tempfile::tempdir().expect("tempdir");
    let config_dir = tempfile::tempdir().expect("tempdir");
    create_fake_cli(path_dir.path(), "my-device");
    set_alias(config_dir.path(), path_dir.path(), "my-device", "mydev");

    let script = generate_zsh(config_dir.path(), path_dir.path());

    // "Alias name to look up" = get alias
    for line in script.lines() {
        if line.contains(":alias_name -- Alias name to look up") {
            assert!(
                line.contains("mydev"),
                "get alias should offer alias name, got: {line}"
            );
            assert!(
                !line.contains("my-device"),
                "get alias should NOT offer CLI name, got: {line}"
            );
        }
    }

    // "Alias name to remove" = unset alias
    for line in script.lines() {
        if line.contains(":alias_name -- Alias name to remove") {
            assert!(
                line.contains("mydev"),
                "unset alias should offer alias name, got: {line}"
            );
            assert!(
                !line.contains("my-device"),
                "unset alias should NOT offer CLI name, got: {line}"
            );
        }
    }
}

// -----------------------------------------------------------------------
// zsh positional arg ordering: cmd before alias_name in set alias
// -----------------------------------------------------------------------

/// In the zsh completion script, `set alias` should have `cmd` (the
/// CLI name with value completions) as the FIRST positional arg and
/// `alias_name` (free-form) as the SECOND. A previous bug had the
/// order reversed.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn zsh_set_alias_cmd_comes_before_alias_name() {
    let path_dir = tempfile::tempdir().expect("tempdir");
    let config_dir = tempfile::tempdir().expect("tempdir");
    create_fake_cli(path_dir.path(), "my-device");

    let script = generate_zsh(config_dir.path(), path_dir.path());

    let mut cmd_line_num = None;
    let mut alias_name_line_num = None;

    for (i, line) in script.lines().enumerate() {
        if line.contains(":cmd -- Target command name") {
            cmd_line_num = Some(i);
        }
        if line.contains(":alias_name -- Alias name to create") {
            alias_name_line_num = Some(i);
        }
    }

    let cmd_pos = cmd_line_num.expect("should find cmd arg in completions");
    let alias_pos = alias_name_line_num.expect("should find alias_name arg in completions");
    assert!(
        cmd_pos < alias_pos,
        "cmd (line {cmd_pos}) should come before alias_name (line {alias_pos})"
    );
}

// -----------------------------------------------------------------------
// Top-level completions include both CLIs and aliases
// -----------------------------------------------------------------------

/// `libiot <TAB>` should show both discovered CLIs and configured
/// aliases as top-level subcommand completions.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn top_level_includes_clis_and_aliases() {
    let path_dir = tempfile::tempdir().expect("tempdir");
    let config_dir = tempfile::tempdir().expect("tempdir");
    create_fake_cli(path_dir.path(), "my-device");
    set_alias(config_dir.path(), path_dir.path(), "my-device", "mydev");

    let script = generate_zsh(config_dir.path(), path_dir.path());

    // Both should appear as subcommand entries (with descriptions).
    assert!(
        script.contains("'my-device:"),
        "top-level should include discovered CLI"
    );
    assert!(script.contains("'mydev:"), "top-level should include alias");
}

// -----------------------------------------------------------------------
// CLI descriptions come from --help, alias descriptions say "Alias for"
// -----------------------------------------------------------------------

/// Discovered CLIs should have their --help title as the completion
/// description. Aliases should say "Alias for <target>".
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn completion_descriptions_are_correct() {
    let path_dir = tempfile::tempdir().expect("tempdir");
    let config_dir = tempfile::tempdir().expect("tempdir");
    create_fake_cli(path_dir.path(), "my-device");
    set_alias(config_dir.path(), path_dir.path(), "my-device", "mydev");

    let script = generate_zsh(config_dir.path(), path_dir.path());

    // CLI description comes from the fake script's --help output
    // (first line of stdout: "Control my-device device").
    assert!(
        script.contains("'my-device:Control my-device device'"),
        "CLI should have --help title as description"
    );

    // Alias description.
    assert!(
        script.contains("'mydev:Alias for my-device'"),
        "alias should have 'Alias for <target>' as description"
    );
}

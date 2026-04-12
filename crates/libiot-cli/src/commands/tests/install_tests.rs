//! Tests for the `install` command handler.
//!
//! Written by Claude Code, reviewed by a human.

use crate::cli::InstallArgs;
use crate::commands::install::build_cargo_install_args;
use crate::commands::install::create_post_install_alias_from;
use crate::error::CliError;
use crate::settings::Settings;
use crate::settings::load_settings_from;
use crate::settings::save_settings_to;

/// Build default [`InstallArgs`] with only the `name` field populated.
fn default_args(name: &str) -> InstallArgs {
    InstallArgs {
        name: name.to_owned(),
        alias: None,
        features: None,
        all_features: false,
        target_dir: None,
        version: None,
        force: false,
        dry_run: false,
        debug: false,
        verbose: false,
        color: None,
        jobs: None,
        quiet: false,
        root: None,
        no_update_completions: true,
    }
}

/// Create a temp dir with a settings file and return the path to
/// `settings.json`.
fn write_settings(settings: &Settings) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("create tempdir");
    let path = dir.path().join("settings.json");
    save_settings_to(settings, &path).expect("write settings");
    (dir, path)
}

// -----------------------------------------------------------------------
// build_cargo_install_args tests
// -----------------------------------------------------------------------

/// Minimal args produce only the crate name.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn build_args_minimal() {
    let args = default_args("foo");
    let result = build_cargo_install_args("libiot-foo-cli", &args, false);
    assert_eq!(result, vec!["libiot-foo-cli"]);
}

/// All optional flags are forwarded to cargo.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn build_args_all_flags() {
    let args = InstallArgs {
        name: "test".to_owned(),
        alias: None,
        features: Some("feat1,feat2".to_owned()),
        all_features: true,
        target_dir: Some("/tmp/target".to_owned()),
        version: Some("1.2.3".to_owned()),
        force: true,
        dry_run: false,
        debug: true,
        verbose: true,
        color: Some("always".to_owned()),
        jobs: Some(4),
        quiet: true,
        root: Some("/usr/local".to_owned()),
        no_update_completions: true,
    };
    let result = build_cargo_install_args("libiot-test-cli", &args, false);
    assert_eq!(
        result,
        vec![
            "libiot-test-cli",
            "--features",
            "feat1,feat2",
            "--all-features",
            "--target-dir",
            "/tmp/target",
            "--version",
            "1.2.3",
            "--force",
            "--debug",
            "--verbose",
            "--color",
            "always",
            "--jobs",
            "4",
            "--quiet",
            "--root",
            "/usr/local",
        ],
    );
}

/// `ctx_quiet` adds `--quiet` even when the arg is not set.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn build_args_ctx_quiet_adds_quiet() {
    let args = default_args("foo");
    let result = build_cargo_install_args("libiot-foo-cli", &args, true);
    assert!(result.contains(&"--quiet".to_owned()));
}

/// When both `args.quiet` and `ctx_quiet` are true, `--quiet` appears
/// exactly once.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn build_args_quiet_not_duplicated() {
    let mut args = default_args("foo");
    args.quiet = true;
    let result = build_cargo_install_args("libiot-foo-cli", &args, true);
    let quiet_count = result.iter().filter(|a| a.as_str() == "--quiet").count();
    assert_eq!(quiet_count, 1, "expected exactly one --quiet flag");
}

/// `--jobs` is forwarded as a string.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn build_args_jobs_forwarded() {
    let mut args = default_args("foo");
    args.jobs = Some(8);
    let result = build_cargo_install_args("libiot-foo-cli", &args, false);
    let jobs_idx = result.iter().position(|a| a == "--jobs").unwrap();
    assert_eq!(result[jobs_idx + 1], "8");
}

// -----------------------------------------------------------------------
// dry-run test
// -----------------------------------------------------------------------

/// Dry-run mode returns `Ok` without spawning cargo.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn dry_run_returns_ok() {
    use crate::commands::install::run_install;
    use crate::output::OutputContext;
    use crate::output::OutputFormat;

    let mut args = default_args("nonexistent-device");
    args.dry_run = true;
    let ctx = OutputContext {
        format: OutputFormat::Human,
        quiet: true,
    };
    let result = run_install(&args, ctx);
    assert!(result.is_ok(), "expected Ok for dry run, got {result:?}");
}

// -----------------------------------------------------------------------
// post-install alias tests
// -----------------------------------------------------------------------

/// Post-install alias creation inserts the alias into settings.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn post_install_alias_creates_alias() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let result = create_post_install_alias_from("shades", "rollease", &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");

    let reloaded = load_settings_from(&path).expect("reload settings");
    assert_eq!(
        reloaded.aliases.get("shades").map(String::as_str),
        Some("rollease"),
    );
}

/// Post-install alias overwrites an existing alias.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn post_install_alias_overwrites_existing() {
    let mut settings = Settings::default();
    settings
        .aliases
        .insert("shades".to_owned(), "old-cmd".to_owned());
    let (_dir, path) = write_settings(&settings);

    let result = create_post_install_alias_from("shades", "new-cmd", &path);
    assert!(result.is_ok(), "expected Ok, got {result:?}");

    let reloaded = load_settings_from(&path).expect("reload settings");
    assert_eq!(
        reloaded.aliases.get("shades").map(String::as_str),
        Some("new-cmd"),
    );
}

/// Post-install alias that shadows a built-in returns
/// `PostInstallAliasFailed`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn post_install_alias_rejects_builtin_shadow() {
    let settings = Settings::default();
    let (_dir, path) = write_settings(&settings);

    let err = create_post_install_alias_from("install", "some-cmd", &path).unwrap_err();
    assert!(
        matches!(err, CliError::PostInstallAliasFailed { .. }),
        "expected PostInstallAliasFailed, got {err:?}",
    );
}

//! Tests for the [`CliError`] enum — exit-code uniqueness, reserved
//! code avoidance, and `kind()` category correctness.

use std::collections::HashSet;

use crate::error::CliError;

/// The total number of `CliError` variants. Keep in sync when adding
/// new variants.
const VARIANT_COUNT: usize = 20;

/// Helper: construct one instance of every `CliError` variant using
/// dummy values.
fn all_variants() -> Vec<CliError> {
    vec![
        CliError::AliasAlreadyExists {
            alias: String::new(),
            target: String::new(),
        },
        CliError::AliasNotFound {
            alias: String::new(),
        },
        CliError::AliasShadowsBuiltin {
            alias: String::new(),
            builtin: String::new(),
        },
        CliError::AliasTargetNotFound { cmd: String::new() },
        CliError::CargoInstallFailed {
            name: String::new(),
            code: 1,
        },
        CliError::CargoSpawnFailed {
            source: std::io::Error::other("test"),
        },
        CliError::CargoUninstallFailed {
            name: String::new(),
            code: 1,
        },
        CliError::DelegationTargetNotFound {
            name: String::new(),
        },
        CliError::EnvVarLibiotPrefix {
            name: String::new(),
        },
        CliError::EnvVarNotFound {
            cmd_or_alias: String::new(),
            name: String::new(),
        },
        CliError::EnvVarTargetNotFound {
            cmd_or_alias: String::new(),
        },
        CliError::ExecFailed {
            name: String::new(),
            source: std::io::Error::other("test"),
        },
        CliError::NotCargoInstalled {
            name: String::new(),
        },
        CliError::NoHomeDir,
        CliError::PostInstallAliasFailed {
            reason: String::new(),
        },
        CliError::SettingsDirError {
            path: String::new(),
            source: std::io::Error::other("test"),
        },
        CliError::SettingsParseError {
            path: String::new(),
            source: serde_json::from_str::<()>("invalid").unwrap_err(),
        },
        CliError::SettingsPermissionError {
            path: String::new(),
            source: std::io::Error::other("test"),
        },
        CliError::SettingsReadError {
            path: String::new(),
            source: std::io::Error::other("test"),
        },
        CliError::SettingsWriteError {
            path: String::new(),
            source: std::io::Error::other("test"),
        },
    ]
}

/// Verifies that every `CliError` variant maps to a unique exit code.
/// If two variants share a code, callers cannot distinguish them by
/// exit status alone.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn every_variant_has_a_unique_exit_code() {
    let variants = all_variants();
    assert_eq!(
        variants.len(),
        VARIANT_COUNT,
        "all_variants() length disagrees with VARIANT_COUNT — \
         did you add a variant without updating both?",
    );

    let codes: HashSet<i32> = variants.iter().map(CliError::exit_code).collect();
    assert_eq!(
        codes.len(),
        VARIANT_COUNT,
        "two or more CliError variants share the same exit code",
    );
}

/// Verifies that no variant uses exit codes 0, 1, or 2. These are
/// reserved: 0 = success, 1 = generic runtime error (catch-all), and
/// 2 = clap usage / argument error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn no_exit_code_is_reserved() {
    let reserved: HashSet<i32> = [0, 1, 2].into_iter().collect();
    for variant in all_variants() {
        let code = variant.exit_code();
        assert!(
            !reserved.contains(&code),
            "{variant:?} uses reserved exit code {code}",
        );
    }
}

/// Verifies that every variant returns a non-empty `kind()` string.
/// An empty kind would break `--output json` error objects.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn every_variant_has_a_non_empty_kind() {
    for variant in all_variants() {
        let kind = variant.kind();
        assert!(!kind.is_empty(), "{variant:?} has an empty kind()",);
    }
}

/// Verifies that every `kind()` value is one of the expected
/// categories. This catches typos and ensures the categories stay
/// within the documented set.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn kind_values_are_from_expected_set() {
    let expected: HashSet<&str> = ["alias", "cargo", "delegation", "env-var", "settings"]
        .into_iter()
        .collect();

    for variant in all_variants() {
        let kind = variant.kind();
        assert!(
            expected.contains(kind),
            "{variant:?} returned kind {kind:?} which is not in {expected:?}",
        );
    }
}

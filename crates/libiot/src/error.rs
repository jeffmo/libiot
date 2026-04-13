//! The [`CliError`] enum, the crate-local [`CliResult`] alias, and
//! exit-code mapping.
//!
//! Every error the CLI can surface is represented here as a variant of
//! [`CliError`]. Each variant carries enough context to produce a
//! human-readable message and maps to a unique, stable process exit
//! code so that scripts can react to specific failure modes.

/// Every error the CLI can surface.
///
/// Variants are in alphabetical order (workspace convention). Exit
/// codes are assigned sequentially starting at 10, in that same
/// alphabetical order.
#[derive(Debug, thiserror::Error)]
pub(crate) enum CliError {
    /// An alias already exists and `--overwrite` was not supplied.
    #[error("alias {alias:?} already points to {target:?} — pass --overwrite to replace it")]
    AliasAlreadyExists {
        /// The alias name the user tried to create.
        alias: String,
        /// The existing target of the alias.
        target: String,
    },

    /// A `get` or `unset` targeted an alias that does not exist.
    #[error("alias {alias:?} not found")]
    AliasNotFound {
        /// The alias name the user referenced.
        alias: String,
    },

    /// The requested alias name collides with a built-in command.
    #[error("alias {alias:?} shadows built-in command {builtin:?}")]
    AliasShadowsBuiltin {
        /// The alias name the user tried to create.
        alias: String,
        /// The built-in command it would shadow.
        builtin: String,
    },

    /// The alias target has no corresponding `libiot-CMD` binary on
    /// `$PATH`.
    #[error("alias target {cmd:?} not found — no libiot-{cmd} on PATH")]
    AliasTargetNotFound {
        /// The target command name.
        cmd: String,
    },

    /// `cargo install` exited with a non-zero status.
    #[error("cargo install {name:?} failed with exit code {code}")]
    CargoInstallFailed {
        /// The crate name passed to `cargo install`.
        name: String,
        /// The exit code returned by the cargo process.
        code: i32,
    },

    /// Could not spawn the `cargo` process at all.
    #[error("could not spawn cargo: {source}")]
    CargoSpawnFailed {
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// `cargo uninstall` exited with a non-zero status.
    #[error("cargo uninstall {name:?} failed with exit code {code}")]
    CargoUninstallFailed {
        /// The crate name passed to `cargo uninstall`.
        name: String,
        /// The exit code returned by the cargo process.
        code: i32,
    },

    /// The delegation name is neither a known alias nor a `libiot-*`
    /// binary on `$PATH`.
    #[error("{name:?} is not a known alias and no libiot-{name} found on PATH")]
    DelegationTargetNotFound {
        /// The name the user invoked.
        name: String,
    },

    /// An environment variable name starts with the reserved `LIBIOT_`
    /// prefix.
    #[error("environment variable {name:?} must not start with LIBIOT_")]
    EnvVarLibiotPrefix {
        /// The rejected variable name.
        name: String,
    },

    /// An `env get` / `env unset` targeted a variable that does not
    /// exist in the settings for the given command or alias.
    #[error("environment variable {name:?} not found for {cmd_or_alias:?}")]
    EnvVarNotFound {
        /// The command or alias whose env store was queried.
        cmd_or_alias: String,
        /// The variable name that was not found.
        name: String,
    },

    /// The target of an `env` subcommand is neither a known alias nor a
    /// `libiot-*` binary on `$PATH`.
    #[error("{cmd_or_alias:?} is not a known alias or PATH binary")]
    EnvVarTargetNotFound {
        /// The command or alias name.
        cmd_or_alias: String,
    },

    /// The `exec` syscall (or equivalent) failed after resolving the
    /// target binary.
    #[error("failed to exec {name:?}: {source}")]
    ExecFailed {
        /// The binary name or path.
        name: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// The target crate was not installed via `cargo install`.
    #[error(
        "{name:?} was not installed via `cargo install` — `libiot update` only works with cargo-installed crates"
    )]
    NotCargoInstalled {
        /// The crate name.
        name: String,
    },

    /// Could not determine the user's home directory.
    #[error("could not determine home directory")]
    NoHomeDir,

    /// `cargo install` succeeded but the post-install alias write
    /// failed.
    #[error("install succeeded but alias creation failed: {reason}")]
    PostInstallAliasFailed {
        /// A human-readable description of the failure.
        reason: String,
    },

    /// Could not create the settings directory.
    #[error("could not create settings directory {path:?}: {source}")]
    SettingsDirError {
        /// The directory path.
        path: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// The settings file contains invalid JSON.
    #[error("invalid JSON in {path:?}: {source}")]
    SettingsParseError {
        /// The file path.
        path: String,
        /// The parse error from `serde_json`.
        source: serde_json::Error,
    },

    /// Could not set permissions on the settings file.
    #[error("could not set permissions on {path:?}: {source}")]
    SettingsPermissionError {
        /// The file path.
        path: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// The settings file exists but could not be read.
    #[error("could not read settings from {path:?}: {source}")]
    SettingsReadError {
        /// The file path.
        path: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Could not write the settings file.
    #[error("could not write settings to {path:?}: {source}")]
    SettingsWriteError {
        /// The file path.
        path: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },
}

impl CliError {
    /// Map to a process exit code.
    ///
    /// Every variant has a unique code starting at 10. Codes 0
    /// (success), 1 (generic runtime), and 2 (clap usage) are
    /// reserved.
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            Self::AliasAlreadyExists { .. } => 10,
            Self::AliasNotFound { .. } => 11,
            Self::AliasShadowsBuiltin { .. } => 12,
            Self::AliasTargetNotFound { .. } => 13,
            Self::CargoInstallFailed { .. } => 14,
            Self::CargoSpawnFailed { .. } => 15,
            Self::CargoUninstallFailed { .. } => 16,
            Self::DelegationTargetNotFound { .. } => 17,
            Self::EnvVarLibiotPrefix { .. } => 18,
            Self::EnvVarNotFound { .. } => 19,
            Self::EnvVarTargetNotFound { .. } => 20,
            Self::ExecFailed { .. } => 21,
            Self::NotCargoInstalled { .. } => 22,
            Self::NoHomeDir => 23,
            Self::PostInstallAliasFailed { .. } => 24,
            Self::SettingsDirError { .. } => 25,
            Self::SettingsParseError { .. } => 26,
            Self::SettingsPermissionError { .. } => 27,
            Self::SettingsReadError { .. } => 28,
            Self::SettingsWriteError { .. } => 29,
        }
    }

    /// A stable, grep-able string identifying the error category.
    /// Used as the `"kind"` field in `--output json` error responses.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::AliasAlreadyExists { .. }
            | Self::AliasNotFound { .. }
            | Self::AliasShadowsBuiltin { .. }
            | Self::AliasTargetNotFound { .. } => "alias",

            Self::CargoInstallFailed { .. }
            | Self::CargoSpawnFailed { .. }
            | Self::CargoUninstallFailed { .. }
            | Self::NotCargoInstalled { .. } => "cargo",

            Self::DelegationTargetNotFound { .. } | Self::ExecFailed { .. } => "delegation",

            Self::EnvVarLibiotPrefix { .. }
            | Self::EnvVarNotFound { .. }
            | Self::EnvVarTargetNotFound { .. } => "env-var",

            Self::NoHomeDir
            | Self::PostInstallAliasFailed { .. }
            | Self::SettingsDirError { .. }
            | Self::SettingsParseError { .. }
            | Self::SettingsPermissionError { .. }
            | Self::SettingsReadError { .. }
            | Self::SettingsWriteError { .. } => "settings",
        }
    }
}

/// Crate-local [`Result`] alias.
pub(crate) type CliResult<T> = std::result::Result<T, CliError>;

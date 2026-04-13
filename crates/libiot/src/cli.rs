//! Top-level clap [`Cli`] struct and the [`Command`] subcommand enum.
//!
//! The `libiot` CLI has two modes: built-in commands (alias management,
//! env-var management, install/uninstall, completions) and delegation
//! to discovered `libiot-*-cli` binaries.  This module defines the
//! clap parse tree for the built-in commands and exposes
//! [`is_builtin`] so that the pre-parse dispatcher can distinguish
//! built-in names from delegated ones.

use clap::Parser;
use clap::Subcommand;

use crate::output::OutputFormat;

// ---------------------------------------------------------------------------
// Built-in command names
// ---------------------------------------------------------------------------

/// All built-in subcommand names (used for alias-collision checking).
pub(crate) const BUILTIN_NAMES: &[&str] = &[
    "completions",
    "config-path",
    "get",
    "help",
    "install",
    "list",
    "set",
    "uninstall",
    "unset",
    "update",
];

/// Check whether a name matches a built-in subcommand.
pub(crate) fn is_builtin(name: &str) -> bool {
    BUILTIN_NAMES.contains(&name)
}

/// Normalize a user-supplied crate name to the short form used
/// internally (e.g. `"rollease-automate-pulse-pro-hub"`).
///
/// Accepts any of:
/// - `"rollease-automate-pulse-pro-hub"` (short form, returned as-is)
/// - `"libiot-rollease-automate-pulse-pro-hub"` (library crate name)
/// - `"libiot-rollease-automate-pulse-pro-hub-cli"` (CLI crate name)
pub(crate) fn normalize_crate_name(name: &str) -> &str {
    if let Some(without_prefix) = name.strip_prefix("libiot-") {
        // Had the libiot- prefix — also strip -cli if present.
        without_prefix
            .strip_suffix("-cli")
            .unwrap_or(without_prefix)
    } else {
        // No libiot- prefix — return as-is. We intentionally do NOT
        // strip a bare -cli suffix (e.g. "foo-cli") because that form
        // is ambiguous without the libiot- prefix.
        name
    }
}

// ---------------------------------------------------------------------------
// Top-level CLI
// ---------------------------------------------------------------------------

/// Unified CLI dispatcher for the libiot ecosystem.
///
/// Discovers all installed `libiot-*-cli` binaries on `$PATH` and
/// exposes them as subcommands, with alias management, per-command
/// environment variable injection, and `cargo install`/`uninstall`
/// wrappers.
#[derive(Parser, Debug)]
#[command(
    name = "libiot",
    version,
    about = "Unified CLI dispatcher for the libiot ecosystem",
    subcommand_required = true,
    arg_required_else_help = true
)]
pub(crate) struct Cli {
    /// Output format: "human" (default) for readable text, "json" for
    /// machine-readable JSON.
    #[arg(long, default_value = "human", global = true)]
    pub format: OutputFormat,

    /// Suppress all non-error output.
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Use verbose output.
    #[arg(long, short, global = true)]
    pub verbose: bool,

    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Command,
}

// ---------------------------------------------------------------------------
// Command enum
// ---------------------------------------------------------------------------

/// Top-level subcommand dispatch for built-in commands.
#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Generate shell completion scripts.
    ///
    /// Run without arguments to see installation instructions for each
    /// supported shell.
    Completions {
        /// Target shell (bash, zsh, fish, powershell, elvish).
        /// Omit to see installation instructions.
        shell: Option<clap_complete::Shell>,
        /// Print only the shell-config snippet (for piping into a config
        /// file). Requires a shell argument.
        #[arg(long)]
        print_config: bool,
    },
    /// Print the path to the settings file.
    ConfigPath,
    /// Query an alias or environment variable.
    Get {
        /// Which item to query.
        #[command(subcommand)]
        target: GetTarget,
    },
    /// Install a libiot CLI crate via cargo install.
    Install(InstallArgs),
    /// List discovered CLIs, aliases, or environment variables.
    List {
        /// Optionally narrow to a specific category.
        #[command(subcommand)]
        target: Option<ListTarget>,
    },
    /// Set an alias or environment variable.
    Set {
        /// Which item to set.
        #[command(subcommand)]
        target: SetTarget,
    },
    /// Uninstall a libiot CLI crate via cargo uninstall.
    Uninstall(UninstallArgs),
    /// Remove an alias or environment variable.
    Unset {
        /// Which item to remove.
        #[command(subcommand)]
        target: UnsetTarget,
    },
    /// Update libiot or an installed CLI to the latest version.
    ///
    /// With no arguments, updates libiot itself. With a CLI name,
    /// updates that specific CLI.
    Update(UpdateArgs),
}

// ---------------------------------------------------------------------------
// SetTarget
// ---------------------------------------------------------------------------

/// Targets for the `set` subcommand.
#[derive(Subcommand, Debug)]
pub(crate) enum SetTarget {
    /// Create an alias for a CLI command name.
    Alias {
        /// Target command name (e.g. "rollease-automate-pulse-pro-hub").
        cmd: String,
        /// Alias name to create.
        alias_name: String,
        /// Overwrite if the alias already exists.
        #[arg(long, short = 'f')]
        overwrite: bool,
    },
    /// Set an environment variable for a command or alias.
    #[command(name = "env-var")]
    EnvVar {
        /// Command or alias name.
        cmd_or_alias: String,
        /// Variable name (without LIBIOT_ prefix).
        var_name: String,
        /// Variable value.
        value: String,
    },
}

// ---------------------------------------------------------------------------
// UnsetTarget
// ---------------------------------------------------------------------------

/// Targets for the `unset` subcommand.
#[derive(Subcommand, Debug)]
pub(crate) enum UnsetTarget {
    /// Remove an alias.
    Alias {
        /// Alias name to remove.
        alias_name: String,
    },
    /// Remove an environment variable.
    #[command(name = "env-var")]
    EnvVar {
        /// Command or alias name.
        cmd_or_alias: String,
        /// Variable name to remove.
        var_name: String,
    },
}

// ---------------------------------------------------------------------------
// GetTarget
// ---------------------------------------------------------------------------

/// Targets for the `get` subcommand.
#[derive(Subcommand, Debug)]
pub(crate) enum GetTarget {
    /// Show the target command for an alias.
    Alias {
        /// Alias name to look up.
        alias_name: String,
    },
    /// Show the value of an environment variable.
    #[command(name = "env-var")]
    EnvVar {
        /// Command or alias name.
        cmd_or_alias: String,
        /// Variable name to look up.
        var_name: String,
    },
}

// ---------------------------------------------------------------------------
// ListTarget
// ---------------------------------------------------------------------------

/// Targets for the `list` subcommand.
#[derive(Subcommand, Debug)]
pub(crate) enum ListTarget {
    /// List all configured aliases.
    Aliases,
    /// List environment variables for all or a specific command/alias.
    #[command(name = "env-vars")]
    EnvVars {
        /// Optionally filter to a specific command or alias.
        cmd_or_alias: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// InstallArgs
// ---------------------------------------------------------------------------

/// Arguments for the `install` subcommand.
#[derive(clap::Args, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct InstallArgs {
    /// Crate to install (e.g. "rollease-automate-pulse-pro-hub").
    pub name: String,
    /// Also create an alias after successful install.
    #[arg(long)]
    pub alias: Option<String>,
    /// Comma-separated list of features to activate.
    #[arg(long)]
    pub features: Option<String>,
    /// Enable all available features.
    #[arg(long)]
    pub all_features: bool,
    /// Directory for all generated artifacts.
    #[arg(long)]
    pub target_dir: Option<String>,
    /// Crate version to install.
    #[arg(long)]
    pub version: Option<String>,
    /// Force overwriting an existing binary.
    #[arg(long, short = 'f')]
    pub force: bool,
    /// Perform all checks without actually installing.
    #[arg(long, short = 'n')]
    pub dry_run: bool,
    /// Build in debug mode (without optimizations).
    #[arg(long)]
    pub debug: bool,
    /// Control when colored output is used.
    #[arg(long)]
    pub color: Option<String>,
    /// Number of parallel jobs for cargo.
    #[arg(long, short)]
    pub jobs: Option<u32>,
    /// Suppress cargo output.
    #[arg(long)]
    pub quiet: bool,
    /// Directory to install packages into.
    #[arg(long)]
    pub root: Option<String>,
    /// Skip regenerating shell completion files after install.
    #[arg(long)]
    pub no_update_completions: bool,
}

// ---------------------------------------------------------------------------
// UninstallArgs
// ---------------------------------------------------------------------------

/// Arguments for the `uninstall` subcommand.
#[derive(clap::Args, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct UninstallArgs {
    /// Installed CLI to remove (e.g. "rollease-automate-pulse-pro-hub").
    pub name: String,
    /// Control when colored output is used.
    #[arg(long)]
    pub color: Option<String>,
    /// Suppress cargo output.
    #[arg(long)]
    pub quiet: bool,
    /// Remove all env vars for this command and its aliases.
    #[arg(long)]
    pub remove_env_vars: bool,
    /// Directory to uninstall packages from.
    #[arg(long)]
    pub root: Option<String>,
    /// Skip regenerating shell completion files after uninstall.
    #[arg(long)]
    pub no_update_completions: bool,
}

// ---------------------------------------------------------------------------
// UpdateArgs
// ---------------------------------------------------------------------------

/// Arguments for the `update` subcommand.
#[derive(clap::Args, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct UpdateArgs {
    /// CLI to update. Omit to update libiot itself.
    pub name: Option<String>,
    /// Update all installed libiot CLIs and libiot itself.
    #[arg(long, short)]
    pub all: bool,
    /// Comma-separated list of features to activate.
    #[arg(long)]
    pub features: Option<String>,
    /// Enable all available features.
    #[arg(long)]
    pub all_features: bool,
    /// Directory for all generated artifacts.
    #[arg(long)]
    pub target_dir: Option<String>,
    /// Perform all checks without actually installing.
    #[arg(long, short = 'n')]
    pub dry_run: bool,
    /// Build in debug mode (without optimizations).
    #[arg(long)]
    pub debug: bool,
    /// Control when colored output is used.
    #[arg(long)]
    pub color: Option<String>,
    /// Number of parallel jobs for cargo.
    #[arg(long, short)]
    pub jobs: Option<u32>,
    /// Suppress cargo output.
    #[arg(long)]
    pub quiet: bool,
    /// Directory to install packages into.
    #[arg(long)]
    pub root: Option<String>,
    /// Skip regenerating shell completion files after update.
    #[arg(long)]
    pub no_update_completions: bool,
}

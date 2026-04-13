//! Output formatting — human-readable and JSON views.
//!
//! The CLI defines its own serializable "view" structs so that the
//! JSON output shape is a deliberate API surface rather than an
//! incidental leak of internal types.  Every command handler calls one
//! of the `render_*` functions, passing the [`OutputContext`] it
//! received from the top-level CLI.

use std::fmt;
use std::path::Path;
use std::str::FromStr;

use crate::error::CliError;

// ---------------------------------------------------------------------------
// OutputFormat
// ---------------------------------------------------------------------------

/// How to format output to stdout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum OutputFormat {
    /// Aligned human-readable text (default).
    #[default]
    Human,
    /// Machine-readable JSON.
    Json,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            other => Err(format!(
                "unknown output format {other:?}: expected \"human\" or \"json\""
            )),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Human => f.write_str("human"),
            Self::Json => f.write_str("json"),
        }
    }
}

// ---------------------------------------------------------------------------
// OutputContext
// ---------------------------------------------------------------------------

/// Bundles the output format with the `--quiet` and `--verbose` flags
/// so that every render function can make all decisions from a single
/// argument.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OutputContext {
    /// The output format requested by the user.
    pub format: OutputFormat,
    /// When `true`, success output is suppressed entirely.
    pub quiet: bool,
    /// When `true`, extra diagnostic output is enabled.
    pub verbose: bool,
}

// ---------------------------------------------------------------------------
// View structs (CLI-local, serde-serializable)
// ---------------------------------------------------------------------------

/// JSON view of a single alias mapping.
#[derive(serde::Serialize)]
pub(crate) struct AliasView<'a> {
    /// The alias name.
    pub alias: &'a str,
    /// The target command the alias resolves to.
    pub target: &'a str,
}

/// JSON view of a discovered CLI binary.
#[derive(serde::Serialize)]
pub(crate) struct CliView<'a> {
    /// The short name (e.g. `rollease-automate-pulse-pro-hub`).
    pub name: &'a str,
    /// The absolute path to the binary on disk.
    pub path: &'a Path,
}

/// JSON view for `list` (all CLIs and aliases).
#[derive(serde::Serialize)]
pub(crate) struct ListAllView<'a> {
    /// Discovered CLI binaries.
    pub clis: Vec<CliView<'a>>,
    /// Configured aliases.
    pub aliases: Vec<AliasView<'a>>,
}

/// JSON view for `list aliases`.
#[derive(serde::Serialize)]
pub(crate) struct ListAliasesView<'a> {
    /// Configured aliases.
    pub aliases: Vec<AliasView<'a>>,
}

/// JSON view of a single environment variable.
#[derive(serde::Serialize)]
pub(crate) struct EnvVarView<'a> {
    /// The variable name.
    pub name: &'a str,
    /// The variable value.
    pub value: &'a str,
}

/// JSON view for `list env-vars`.
#[derive(serde::Serialize)]
pub(crate) struct ListEnvVarsView<'a> {
    /// The command or alias whose env vars are listed.
    pub cmd_or_alias: &'a str,
    /// The environment variables configured for the command.
    pub env_vars: Vec<EnvVarView<'a>>,
    /// If the target was an alias, the command it resolved to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_from: Option<&'a str>,
}

/// JSON view of a single environment variable's value (for `get`).
#[derive(serde::Serialize)]
pub(crate) struct EnvVarValueView<'a> {
    /// The variable name.
    pub name: &'a str,
    /// The variable value.
    pub value: &'a str,
}

/// JSON view of a `cargo install` or `cargo uninstall` result.
#[derive(serde::Serialize)]
pub(crate) struct CargoResultView<'a> {
    /// Whether the cargo operation succeeded.
    pub ok: bool,
    /// The crate that was installed or uninstalled.
    pub crate_name: &'a str,
    /// Captured cargo stdout/stderr, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo_output: Option<&'a str>,
    /// The alias that was auto-created after install, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias_created: Option<&'a str>,
}

// ---------------------------------------------------------------------------
// Render helpers — success output
// ---------------------------------------------------------------------------

/// Render a simple success message to stdout.
///
/// Human: prints the message as-is. JSON: `{"ok":true,"message":"..."}`.
pub(crate) fn render_ok_message(message: &str, ctx: OutputContext) {
    if ctx.quiet {
        return;
    }
    match ctx.format {
        OutputFormat::Human => println!("{message}"),
        OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct OkMsg<'a> {
                ok: bool,
                message: &'a str,
            }
            let view = OkMsg { ok: true, message };
            println!(
                "{}",
                serde_json::to_string_pretty(&view).expect("OkMsg is serializable")
            );
        },
    }
}

/// Render a single alias mapping to stdout.
///
/// Human: prints only the target. JSON: `{"alias":"...","target":"..."}`.
pub(crate) fn render_alias(alias: &str, target: &str, ctx: OutputContext) {
    if ctx.quiet {
        return;
    }
    match ctx.format {
        OutputFormat::Human => println!("{target}"),
        OutputFormat::Json => {
            let view = AliasView { alias, target };
            println!(
                "{}",
                serde_json::to_string_pretty(&view).expect("AliasView is serializable")
            );
        },
    }
}

/// Render a single environment variable value to stdout.
///
/// Human: prints only the value. JSON: `{"name":"...","value":"..."}`.
pub(crate) fn render_env_var_value(name: &str, value: &str, ctx: OutputContext) {
    if ctx.quiet {
        return;
    }
    match ctx.format {
        OutputFormat::Human => println!("{value}"),
        OutputFormat::Json => {
            let view = EnvVarValueView { name, value };
            println!(
                "{}",
                serde_json::to_string_pretty(&view).expect("EnvVarValueView is serializable")
            );
        },
    }
}

/// Render the full `list` output (CLIs + aliases) to stdout.
///
/// Human format:
/// ```text
/// Installed CLIs:
///   name-a    /path/to/libiot-name-a
///   name-b    /path/to/libiot-name-b
///
/// Aliases:
///   shades -> rollease-automate-pulse-pro-hub
/// ```
pub(crate) fn render_list_all(clis: &[CliView<'_>], aliases: &[AliasView<'_>], ctx: OutputContext) {
    if ctx.quiet {
        return;
    }
    match ctx.format {
        OutputFormat::Human => {
            println!("Installed CLIs:");
            if clis.is_empty() {
                println!("  (none)");
            } else {
                let name_w = clis.iter().map(|c| c.name.len()).max().unwrap_or(0);
                for cli in clis {
                    println!("  {:<name_w$}    {}", cli.name, cli.path.display(),);
                }
            }

            println!();
            println!("Aliases:");
            if aliases.is_empty() {
                println!("  (none)");
            } else {
                for a in aliases {
                    println!("  {} -> {}", a.alias, a.target);
                }
            }
        },
        OutputFormat::Json => {
            let view = ListAllView {
                clis: clis.to_vec_views(),
                aliases: aliases.to_vec_views(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&view).expect("ListAllView is serializable")
            );
        },
    }
}

/// Render the `list aliases` output to stdout.
pub(crate) fn render_list_aliases(aliases: &[AliasView<'_>], ctx: OutputContext) {
    if ctx.quiet {
        return;
    }
    match ctx.format {
        OutputFormat::Human => {
            if aliases.is_empty() {
                println!("(no aliases configured)");
            } else {
                for a in aliases {
                    println!("{} -> {}", a.alias, a.target);
                }
            }
        },
        OutputFormat::Json => {
            let view = ListAliasesView {
                aliases: aliases.to_vec_views(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&view).expect("ListAliasesView is serializable")
            );
        },
    }
}

/// Render the `list env-vars` output to stdout.
///
/// Human format:
/// ```text
/// Environment variables for "shades" (alias for "rollease-automate-pulse-pro-hub"):
///   LIBIOT_PULSE_PRO_HUB=192.168.1.1
///   LIBIOT_FOO=bar
/// ```
pub(crate) fn render_list_env_vars(
    cmd_or_alias: &str,
    env_vars: &[EnvVarView<'_>],
    resolved_from: Option<&str>,
    ctx: OutputContext,
) {
    if ctx.quiet {
        return;
    }
    match ctx.format {
        OutputFormat::Human => {
            let suffix = match resolved_from {
                Some(origin) => {
                    format!(" (alias for {origin:?})")
                },
                None => String::new(),
            };
            println!("Environment variables for {cmd_or_alias:?}{suffix}:");
            if env_vars.is_empty() {
                println!("  (none)");
            } else {
                for ev in env_vars {
                    println!("  {}={}", ev.name, ev.value);
                }
            }
        },
        OutputFormat::Json => {
            let view = ListEnvVarsView {
                cmd_or_alias,
                env_vars: env_vars.to_vec_views(),
                resolved_from,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&view).expect("ListEnvVarsView is serializable")
            );
        },
    }
}

/// Render a `cargo install` / `cargo uninstall` result to stdout.
pub(crate) fn render_cargo_result(view: &CargoResultView<'_>, ctx: OutputContext) {
    if ctx.quiet {
        return;
    }
    match ctx.format {
        OutputFormat::Human => {
            if view.ok {
                println!("ok: {}", view.crate_name);
            } else {
                println!("failed: {}", view.crate_name);
            }
            if let Some(output) = view.cargo_output {
                println!("{output}");
            }
            if let Some(alias) = view.alias_created {
                println!("alias created: {alias}");
            }
        },
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(view).expect("CargoResultView is serializable")
            );
        },
    }
}

/// Render the config file path to stdout.
///
/// Human: prints the path as-is. JSON: `{"config_path":"..."}`.
pub(crate) fn render_config_path(path: &Path, ctx: OutputContext) {
    if ctx.quiet {
        return;
    }
    match ctx.format {
        OutputFormat::Human => println!("{}", path.display()),
        OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct ConfigPath<'a> {
                config_path: &'a Path,
            }
            let view = ConfigPath { config_path: path };
            println!(
                "{}",
                serde_json::to_string_pretty(&view).expect("ConfigPath is serializable")
            );
        },
    }
}

// ---------------------------------------------------------------------------
// Error reporting
// ---------------------------------------------------------------------------

/// Report a [`CliError`] to stderr.
///
/// This function is **not** affected by `quiet` — errors are always
/// printed so that failures are never silently swallowed.
///
/// - JSON: `{"error":"...","kind":"...","code":N}` on stderr.
/// - Human: `error: <message>` on stderr.
pub(crate) fn report_error(err: &CliError, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct ErrView<'a> {
                error: String,
                kind: &'a str,
                code: i32,
            }
            let view = ErrView {
                error: err.to_string(),
                kind: err.kind(),
                code: err.exit_code(),
            };
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&view).expect("ErrView is serializable")
            );
        },
        OutputFormat::Human => {
            eprintln!("error: {err}");
        },
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Convenience trait to clone a `&[ViewStruct<'a>]` slice into a
/// `Vec<ViewStruct<'a>>` for embedding in an outer view struct.
///
/// We cannot derive `Clone` on the view structs (they hold `&Path`
/// which is fine to copy but `#[derive(Clone)]` on a struct with
/// `serde::Serialize` would need `Clone` on every field type). Instead
/// we just rebuild the vec from the slice references, which is
/// trivially cheap for these tiny views.
trait ToVecViews<'a, T> {
    /// Rebuild a `Vec<T>` from a shared slice.
    fn to_vec_views(&self) -> Vec<T>;
}

impl<'a> ToVecViews<'a, CliView<'a>> for &[CliView<'a>] {
    fn to_vec_views(&self) -> Vec<CliView<'a>> {
        self.iter()
            .map(|c| CliView {
                name: c.name,
                path: c.path,
            })
            .collect()
    }
}

impl<'a> ToVecViews<'a, AliasView<'a>> for &[AliasView<'a>] {
    fn to_vec_views(&self) -> Vec<AliasView<'a>> {
        self.iter()
            .map(|a| AliasView {
                alias: a.alias,
                target: a.target,
            })
            .collect()
    }
}

impl<'a> ToVecViews<'a, EnvVarView<'a>> for &[EnvVarView<'a>] {
    fn to_vec_views(&self) -> Vec<EnvVarView<'a>> {
        self.iter()
            .map(|e| EnvVarView {
                name: e.name,
                value: e.value,
            })
            .collect()
    }
}

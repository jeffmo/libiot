//! Destructive hub operations — the `dangerous` subcommand group.
//!
//! This entire module is compiled only when the `dangerous-ops` Cargo
//! feature is enabled. The feature forwards to the library crate's
//! own `dangerous-ops` feature so both halves are enabled with a
//! single `--features dangerous-ops` at install time.

use crate::cli::DangerousOp;
use crate::commands::Hub;
use crate::error::CliResult;
use crate::output::OutputFormat;
use crate::output::render_ok;

/// Dispatch a `dangerous <op>` subcommand.
pub(crate) async fn run_dangerous(hub: &Hub, op: DangerousOp, fmt: OutputFormat) -> CliResult<()> {
    match op {
        DangerousOp::Pair => {
            hub.pair_motor().await?;
            render_ok(fmt);
        },
        DangerousOp::Unpair { motor } => {
            let addr = motor.resolve(hub).await?;
            hub.unpair_motor(&addr).await?;
            render_ok(fmt);
        },
        DangerousOp::Delete { motor } => {
            let addr = motor.resolve(hub).await?;
            hub.delete_motor(&addr).await?;
            render_ok(fmt);
        },
        DangerousOp::FactoryReset => {
            hub.factory_reset().await?;
            render_ok(fmt);
        },
        DangerousOp::SetUpperLimit { motor } => {
            let addr = motor.resolve(hub).await?;
            hub.set_upper_limit(&addr).await?;
            render_ok(fmt);
        },
        DangerousOp::SetLowerLimit { motor } => {
            let addr = motor.resolve(hub).await?;
            hub.set_lower_limit(&addr).await?;
            render_ok(fmt);
        },
        DangerousOp::DeleteLimits { motor } => {
            let addr = motor.resolve(hub).await?;
            hub.delete_limits(&addr).await?;
            render_ok(fmt);
        },
    }
    Ok(())
}

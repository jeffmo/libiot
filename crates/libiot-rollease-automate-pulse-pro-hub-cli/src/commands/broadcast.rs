//! Broadcast control handlers (open-all, close-all, stop-all).

use crate::commands::Hub;
use crate::error::CliResult;
use crate::output::OutputFormat;
use crate::output::render_ok;

/// `open-all` — open every paired motor.
pub(crate) async fn run_open_all(hub: &Hub, fmt: OutputFormat) -> CliResult<()> {
    hub.open_all().await?;
    render_ok(fmt);
    Ok(())
}

/// `close-all` — close every paired motor.
pub(crate) async fn run_close_all(hub: &Hub, fmt: OutputFormat) -> CliResult<()> {
    hub.close_all().await?;
    render_ok(fmt);
    Ok(())
}

/// `stop-all` — stop every paired motor.
pub(crate) async fn run_stop_all(hub: &Hub, fmt: OutputFormat) -> CliResult<()> {
    hub.stop_all().await?;
    render_ok(fmt);
    Ok(())
}

//! Hub-level query handlers (`hub info`, `hub name`, `hub serial`).

use crate::commands::Hub;
use crate::error::CliResult;
use crate::output::OutputFormat;
use crate::output::render_hub_info;
use crate::output::render_string;

/// `hub info` — query full hub info (name, serial, all motors).
pub(crate) async fn run_info(hub: &Hub, fmt: OutputFormat) -> CliResult<()> {
    let info = hub.info().await?;
    render_hub_info(&info, fmt);
    Ok(())
}

/// `hub name` — query the hub's friendly name.
pub(crate) async fn run_name(hub: &Hub, fmt: OutputFormat) -> CliResult<()> {
    let name = hub.hub_name().await?;
    render_string(&name, fmt);
    Ok(())
}

/// `hub serial` — query the hub's serial number.
pub(crate) async fn run_serial(hub: &Hub, fmt: OutputFormat) -> CliResult<()> {
    let serial = hub.hub_serial().await?;
    render_string(&serial, fmt);
    Ok(())
}

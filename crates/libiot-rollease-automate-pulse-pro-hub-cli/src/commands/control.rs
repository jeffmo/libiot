//! Per-motor control handlers (open, close, stop, set-position, set-tilt,
//! jog-open, jog-close).
//!
//! All motor control commands are fire-and-forget — they return as soon
//! as the command has been written to the hub. The shade may still be
//! moving when the CLI exits. See §2.9 quirk #4 of
//! `PULSE_PRO_LOCAL_API.md`.

use crate::commands::Hub;
use crate::error::CliResult;
use crate::motor_selector::MotorSelector;
use crate::output::OutputFormat;
use crate::output::render_ok;

/// `open <motor>` — fully open (lift to 0% closed).
pub(crate) async fn run_open(hub: &Hub, motor: MotorSelector, fmt: OutputFormat) -> CliResult<()> {
    let addr = motor.resolve(hub).await?;
    hub.open(&addr).await?;
    render_ok(fmt);
    Ok(())
}

/// `close <motor>` — fully close (lift to 100% closed).
pub(crate) async fn run_close(hub: &Hub, motor: MotorSelector, fmt: OutputFormat) -> CliResult<()> {
    let addr = motor.resolve(hub).await?;
    hub.close(&addr).await?;
    render_ok(fmt);
    Ok(())
}

/// `stop <motor>` — stop any in-flight movement.
pub(crate) async fn run_stop(hub: &Hub, motor: MotorSelector, fmt: OutputFormat) -> CliResult<()> {
    let addr = motor.resolve(hub).await?;
    hub.stop(&addr).await?;
    render_ok(fmt);
    Ok(())
}

/// `set-position <motor> <pct>` — move to a specific closed-lift
/// percentage.
pub(crate) async fn run_set_position(
    hub: &Hub,
    motor: MotorSelector,
    percent: u8,
    fmt: OutputFormat,
) -> CliResult<()> {
    let addr = motor.resolve(hub).await?;
    hub.set_position(&addr, percent).await?;
    render_ok(fmt);
    Ok(())
}

/// `set-tilt <motor> <pct>` — set slat tilt angle on venetian blinds.
pub(crate) async fn run_set_tilt(
    hub: &Hub,
    motor: MotorSelector,
    percent: u8,
    fmt: OutputFormat,
) -> CliResult<()> {
    let addr = motor.resolve(hub).await?;
    hub.set_tilt(&addr, percent).await?;
    render_ok(fmt);
    Ok(())
}

/// `jog-open <motor>` — nudge one step open.
pub(crate) async fn run_jog_open(
    hub: &Hub,
    motor: MotorSelector,
    fmt: OutputFormat,
) -> CliResult<()> {
    let addr = motor.resolve(hub).await?;
    hub.jog_open(&addr).await?;
    render_ok(fmt);
    Ok(())
}

/// `jog-close <motor>` — nudge one step closed.
pub(crate) async fn run_jog_close(
    hub: &Hub,
    motor: MotorSelector,
    fmt: OutputFormat,
) -> CliResult<()> {
    let addr = motor.resolve(hub).await?;
    hub.jog_close(&addr).await?;
    render_ok(fmt);
    Ok(())
}

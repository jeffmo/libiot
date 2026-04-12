//! Per-motor query handlers (`motor <motor> [name|position|version|voltage]`).

use crate::cli::MotorQuery;
use crate::commands::Hub;
use crate::error::CliResult;
use crate::motor_selector::MotorSelector;
use crate::output::OutputFormat;
use crate::output::render_hub_info;
use crate::output::render_motor_position;
use crate::output::render_motor_version;
use crate::output::render_motor_voltage;
use crate::output::render_string;

/// Dispatch a `motor <motor> [query]` subcommand.
///
/// When `query` is `None`, the user typed just `motor <motor>` with no
/// sub-query. In that case, show all available info for the motor by
/// calling `hub.info()` and filtering to the target motor.
pub(crate) async fn run_motor_query(
    hub: &Hub,
    motor: MotorSelector,
    query: Option<MotorQuery>,
    fmt: OutputFormat,
) -> CliResult<()> {
    let addr = motor.resolve(hub).await?;

    match query {
        None => {
            // Show all info for one motor. Use hub.info() to get
            // everything in one batched round-trip, then filter to the
            // target motor.
            let mut info = hub.info().await?;
            info.motors.retain(|m| m.address == addr);
            render_hub_info(&info, fmt);
            Ok(())
        },
        Some(MotorQuery::Name) => {
            let name = hub.motor_name(&addr).await?;
            render_string(&name, fmt);
            Ok(())
        },
        Some(MotorQuery::Position) => {
            let pos = hub.motor_position(&addr).await?;
            render_motor_position(pos, fmt);
            Ok(())
        },
        Some(MotorQuery::Version) => {
            let ver = hub.motor_version(&addr).await?;
            render_motor_version(&ver, fmt);
            Ok(())
        },
        Some(MotorQuery::Voltage) => {
            let volt = hub.motor_voltage(&addr).await?;
            render_motor_voltage(volt, fmt);
            Ok(())
        },
    }
}

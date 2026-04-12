//! Subcommand dispatch and handler modules.

mod broadcast;
mod completions;
mod control;
#[cfg(feature = "dangerous-ops")]
mod dangerous;
mod hub;
mod motor;

use libiot_rollease_automate_pulse_pro_hub::AutomatePulseProHub;

use crate::cli::Cli;
use crate::cli::Command;
use crate::cli::HubQuery;
use crate::error::CliResult;
use crate::hub_connection::connect_from_cli;

/// Execute the CLI command. This is the single entry point called from
/// `main()` after clap parsing.
pub(crate) async fn run(cli: Cli) -> CliResult<()> {
    let fmt = cli.format;

    match cli.command {
        // Shell completions and man pages don't need a hub connection.
        Command::Completions { shell } => {
            completions::run_completions(shell);
            return Ok(());
        },
        Command::Man { path } => {
            completions::run_man_page(&path);
            return Ok(());
        },
        _ => {},
    }

    // Everything else needs a hub connection.
    let hub = connect_from_cli(cli.hub.as_deref()).await?;

    match cli.command {
        // Motor control (flat).
        Command::Open { motor } => control::run_open(&hub, motor, fmt).await,
        Command::Close { motor } => control::run_close(&hub, motor, fmt).await,
        Command::Stop { motor } => control::run_stop(&hub, motor, fmt).await,
        Command::SetPosition { motor, percent } => {
            control::run_set_position(&hub, motor, percent, fmt).await
        },
        Command::SetTilt { motor, percent } => {
            control::run_set_tilt(&hub, motor, percent, fmt).await
        },
        Command::JogOpen { motor } => control::run_jog_open(&hub, motor, fmt).await,
        Command::JogClose { motor } => control::run_jog_close(&hub, motor, fmt).await,

        // Broadcast.
        Command::OpenAll => broadcast::run_open_all(&hub, fmt).await,
        Command::CloseAll => broadcast::run_close_all(&hub, fmt).await,
        Command::StopAll => broadcast::run_stop_all(&hub, fmt).await,

        // Hub queries.
        Command::Hub { query } => match query {
            HubQuery::Info => hub::run_info(&hub, fmt).await,
            HubQuery::Name => hub::run_name(&hub, fmt).await,
            HubQuery::Serial => hub::run_serial(&hub, fmt).await,
        },

        // Per-motor queries.
        Command::Motor { motor, query } => motor::run_motor_query(&hub, motor, query, fmt).await,

        // Dangerous ops.
        #[cfg(feature = "dangerous-ops")]
        Command::Dangerous { op } => dangerous::run_dangerous(&hub, op, fmt).await,

        // Already handled above (completions, man).
        Command::Completions { .. } | Command::Man { .. } => unreachable!(),
    }
}

/// A thin helper re-exported for handler modules that need a hub ref
/// without importing the full connection logic.
pub(crate) type Hub = AutomatePulseProHub;

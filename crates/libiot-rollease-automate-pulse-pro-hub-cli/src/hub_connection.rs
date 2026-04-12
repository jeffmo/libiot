//! Hub address parsing and connection establishment.

use libiot_rollease_automate_pulse_pro_hub::AutomatePulseProHub;
use libiot_rollease_automate_pulse_pro_hub::DEFAULT_PORT;

use crate::error::CliError;
use crate::error::CliResult;

/// Parse a `HOST[:PORT]` string into a connectable address, applying
/// the default port when `:PORT` is omitted.
///
/// Handles:
/// - `192.168.5.234` → `192.168.5.234:1487`
/// - `192.168.5.234:9999` → `192.168.5.234:9999`
/// - `my-hub.local` → `my-hub.local:1487`
/// - `my-hub.local:9999` → `my-hub.local:9999`
/// - `[::1]` → `[::1]:1487`
/// - `[::1]:9999` → `[::1]:9999`
///
/// Bare IPv6 without brackets (e.g. `::1`) is rejected because the
/// colon in the address is ambiguous with the port separator.
pub(crate) fn parse_hub_spec(input: &str) -> CliResult<String> {
    let input = input.trim();

    if input.is_empty() {
        return Err(CliError::InvalidHubAddress {
            input: input.to_owned(),
            reason: "address is empty".to_owned(),
        });
    }

    // Bracketed IPv6: [::1] or [::1]:port
    if input.starts_with('[') {
        let close = input.find(']').ok_or_else(|| CliError::InvalidHubAddress {
            input: input.to_owned(),
            reason: "missing closing ']' for IPv6 address".to_owned(),
        })?;
        let after_bracket = &input[close + 1..];
        if after_bracket.is_empty() {
            // [::1] with no port → append default
            return Ok(format!("{input}:{DEFAULT_PORT}"));
        }
        if after_bracket.starts_with(':') {
            // [::1]:port → already has a port
            return Ok(input.to_owned());
        }
        return Err(CliError::InvalidHubAddress {
            input: input.to_owned(),
            reason: format!("unexpected characters after ']': {after_bracket:?}"),
        });
    }

    // Reject bare IPv6 (contains more than one colon without brackets).
    if input.chars().filter(|&c| c == ':').count() > 1 {
        return Err(CliError::InvalidHubAddress {
            input: input.to_owned(),
            reason: "bare IPv6 addresses must be enclosed in brackets, e.g. [::1]".to_owned(),
        });
    }

    // IPv4 or hostname, possibly with :port.
    if let Some(colon_idx) = input.rfind(':') {
        // Check if the part after the last colon is a valid port number.
        let port_part = &input[colon_idx + 1..];
        if port_part.parse::<u16>().is_ok() {
            // host:port already present
            return Ok(input.to_owned());
        }
    }

    // No port → append default.
    Ok(format!("{input}:{DEFAULT_PORT}"))
}

/// Resolve the `--hub` flag (already env-var-resolved by clap) into
/// a connected [`AutomatePulseProHub`].
pub(crate) async fn connect_from_cli(hub_flag: Option<&str>) -> CliResult<AutomatePulseProHub> {
    let spec = hub_flag.ok_or(CliError::NoHubAddress)?;
    let resolved = parse_hub_spec(spec)?;
    Ok(AutomatePulseProHub::connect(resolved).await?)
}

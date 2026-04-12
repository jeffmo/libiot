//! Motor identification — by 3-char address or by friendly name.

use libiot_rollease_automate_pulse_pro_hub::AutomatePulseProHub;
use libiot_rollease_automate_pulse_pro_hub::HubInfo;
use libiot_rollease_automate_pulse_pro_hub::MotorAddress;

use crate::error::CliError;
use crate::error::CliResult;

/// A motor identifier as typed by the user: either an exact 3-char
/// address or a friendly name that will be resolved via a hub lookup.
#[derive(Clone, Debug)]
pub(crate) enum MotorSelector {
    /// A valid 3-character address. Passed directly to the hub with no
    /// extra round-trip.
    Exact(MotorAddress),
    /// A friendly-name substring. Resolved against the hub's paired
    /// motor list via case-insensitive substring matching.
    Name(String),
}

impl std::str::FromStr for MotorSelector {
    type Err = std::convert::Infallible;

    /// Parse the user's input into either `Exact` (if it's a valid
    /// 3-char alphanumeric address) or `Name` (anything else). When
    /// the input is both a valid address AND a substring of a motor
    /// name, the address interpretation wins — this is documented in
    /// `--help`.
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Ok(addr) = MotorAddress::new(s) {
            return Ok(Self::Exact(addr));
        }
        Ok(Self::Name(s.to_owned()))
    }
}

impl std::fmt::Display for MotorSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact(addr) => f.write_str(addr.as_str()),
            Self::Name(name) => f.write_str(name),
        }
    }
}

impl MotorSelector {
    /// Resolve this selector to a concrete [`MotorAddress`].
    ///
    /// `Exact` returns immediately. `Name` calls `hub.info()` and
    /// runs case-insensitive substring matching against every paired
    /// motor's friendly name.
    pub(crate) async fn resolve(&self, hub: &AutomatePulseProHub) -> CliResult<MotorAddress> {
        match self {
            Self::Exact(addr) => Ok(*addr),
            Self::Name(name) => {
                let info = hub.info().await?;
                resolve_against_hub_info(name, &info)
            },
        }
    }
}

/// Pure function that resolves a name against a [`HubInfo`]. Extracted
/// from [`MotorSelector::resolve`] so it can be unit-tested with a
/// hand-crafted `HubInfo` without touching the network.
pub(crate) fn resolve_against_hub_info(name: &str, info: &HubInfo) -> CliResult<MotorAddress> {
    let lower_name = name.to_ascii_lowercase();

    let matches: Vec<_> = info
        .motors
        .iter()
        .filter(|m| {
            m.name
                .as_deref()
                .is_some_and(|n| n.to_ascii_lowercase().contains(&lower_name))
        })
        .collect();

    match matches.len() {
        0 => {
            let candidates = info
                .motors
                .iter()
                .map(|m| format!("{} ({})", m.address, m.name.as_deref().unwrap_or("?"),))
                .collect();
            Err(CliError::MotorNameNoMatch {
                name: name.to_owned(),
                candidates,
            })
        },
        1 => Ok(matches[0].address),
        _ => {
            let candidates = matches
                .iter()
                .map(|m| format!("{} ({})", m.address, m.name.as_deref().unwrap_or("?"),))
                .collect();
            Err(CliError::MotorNameAmbiguous {
                name: name.to_owned(),
                candidates,
            })
        },
    }
}

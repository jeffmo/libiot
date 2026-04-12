//! The [`HubInfo`] struct — full hub state returned by `info()`.

use crate::motor::Motor;

/// Current state of the hub and all its paired motors.
///
/// Assembled by [`crate::AutomatePulseProHub::info`] using the
/// one-shot batch-query pattern documented in §3.1 of the in-crate
/// `PULSE_PRO_LOCAL_API.md`: a single TCP write containing the hub
/// name, hub serial, version enumeration, and position enumeration
/// queries, followed by a second batched write for per-motor friendly
/// names.
///
/// Motors that were enumerated but did not respond to the position
/// query (because they are offline or out of RF range) are still
/// present in `motors` — their [`Motor::position`] field is `None`.
/// Motors that never responded to enumeration at all do not appear in
/// `motors`, which matches the behavior of the `blinds.sh list`
/// reference implementation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HubInfo {
    /// The hub's friendly name as set in the Pulse 2 mobile app.
    pub hub_name: String,

    /// The hub's serial number.
    pub hub_serial: String,

    /// The list of paired motors discovered via the broadcast enumeration
    /// query, with `HubGateway` type entries filtered out. Order matches
    /// the order the hub reported the motors in.
    pub motors: Vec<Motor>,
}

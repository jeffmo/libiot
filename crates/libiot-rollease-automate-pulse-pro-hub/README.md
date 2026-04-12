# libiot-rollease-automate-pulse-pro-hub

Async Rust client for the **Rollease Acmeda Automate Pulse Pro** shade
hub, speaking the local LAN ASCII protocol on TCP port 1487. No auth,
no cloud, no TLS.

Part of the [`libiot`](../..) workspace — a collection of Rust
connector libraries for consumer IoT devices.

## Quick start

```rust,no_run
use libiot_rollease_automate_pulse_pro_hub::AutomatePulseProHub;
use libiot_rollease_automate_pulse_pro_hub::MotorAddress;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let hub = AutomatePulseProHub::connect("192.168.5.234:1487").await?;

// Close every shade in the house.
hub.close_all().await?;

// Move one specific shade to 50% closed.
let kitchen = MotorAddress::new("3YC")?;
hub.set_position(&kitchen, 50).await?;

// Query the full hub info — name, serial, and every paired motor.
let info = hub.info().await?;
println!("hub: {} (serial {})", info.hub_name, info.hub_serial);
for motor in &info.motors {
    println!("  {}: {:?}", motor.address, motor.position);
}
# Ok(()) }
```

## Protocol reference

The primary protocol reference consulted by this crate is the in-crate
copy of [`PULSE_PRO_LOCAL_API.md`](./PULSE_PRO_LOCAL_API.md), which
combines the official Rollease Acmeda *Automate Pulse 2 LINQ*
integrator PDF, the open-source
[`aiopulse2`](https://github.com/sillyfrog/aiopulse2) Python library,
and direct probing of a real hub. Read that document if you're
extending this crate — especially the sections on the field-verified
position response format (§2.5) and the full list of hub error codes
(§2.8).

## Feature flags

- `dangerous-ops` — exposes destructive protocol operations (pair
  motor, unpair, delete motor, factory reset, set/delete limits).
  Off by default because a typo here can wipe the hub's pairing
  configuration. Opt in only if you understand exactly what you're
  doing.

## Status and future work

See [`project-tracker.md`](./project-tracker.md) for the current
state, planned follow-ups, and open questions. Notable items not yet
implemented:

- WebSocket RPC channel (TCP/443, TLS) — rooms, scenes, timers, and
  real-time position push updates. The ASCII channel this crate uses
  covers all motor control and queries but not these features.
- Case-insensitive friendly-name resolution as a convenience wrapper
  over `snapshot()`.
- Automatic reconnect after a dropped TCP connection.

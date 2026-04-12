# libiot-rollease-automate-pulse-pro-hub-cli

Command-line interface for the **Rollease Acmeda Automate Pulse Pro**
shade hub. Wraps every operation in the
[`libiot-rollease-automate-pulse-pro-hub`](../libiot-rollease-automate-pulse-pro-hub/)
library as an ergonomic CLI subcommand so you can control your shades
from a terminal or script without writing Rust.

Part of the [`libiot`](../..) workspace — a collection of Rust
connector libraries for consumer IoT devices.

## Install

```bash
cargo install libiot-rollease-automate-pulse-pro-hub-cli

# Optional: enable destructive operations (pair, unpair, factory reset, etc.)
cargo install libiot-rollease-automate-pulse-pro-hub-cli --features dangerous-ops
```

The installed binary name is `libiot-rollease-automate-pulse-pro-hub`
(without the `-cli` suffix).

## Quick start

Set the hub address once so you don't have to repeat it:

```bash
export LIBIOT_PULSE_PRO_HUB=192.168.5.234
```

Then:

```bash
# Close the kitchen shade by friendly name
libiot-rollease-automate-pulse-pro-hub close kitchen

# Query full hub info — name, serial, all paired motors with positions
libiot-rollease-automate-pulse-pro-hub hub info

# Query a specific motor's position by its 3-char address
libiot-rollease-automate-pulse-pro-hub motor 3YC position

# Move a shade to 50% closed
libiot-rollease-automate-pulse-pro-hub set-position "dining room" 50

# Get JSON output for scripting
libiot-rollease-automate-pulse-pro-hub --output json hub info
```

## Hub address

Pass the hub's LAN IP (and optionally a port) via:

- `--hub HOST[:PORT]` on the command line, or
- `LIBIOT_PULSE_PRO_HUB` environment variable

Port defaults to 1487 (the Pulse Pro ASCII protocol port) if omitted.
The hub must have been paired once via the Automate Pulse 2 mobile app
before it accepts TCP connections on port 1487.

## Motor identification

Every subcommand that targets a specific motor accepts either:

- A **3-character address** (e.g. `4JK`, `MWX`, `3YC`) — passed
  directly to the hub with no extra round-trip.
- A **friendly name** (e.g. `kitchen`, `dining room`) — resolved via a
  case-insensitive substring match against all paired motor names. This
  requires one extra hub round-trip for name resolution. If the name
  matches zero or more than one motor, an error is printed listing all
  available motors.

When a 3-character input is both a valid address AND a substring of a
motor name, the address interpretation wins.

## Feature flags

- `dangerous-ops` — enables the `dangerous` subcommand group, which
  exposes destructive protocol operations: pair motor, unpair, delete,
  factory reset, set/delete upper and lower limits. Off by default
  because a typo here can wipe the hub's pairing configuration.

## Subcommands

### Motor control

| Subcommand                      | Description                                      |
| ------------------------------- | ------------------------------------------------ |
| `open <motor>`                  | Fully open (lift to 0% closed)                   |
| `close <motor>`                 | Fully close (lift to 100% closed)                |
| `stop <motor>`                  | Stop any in-flight movement                      |
| `set-position <motor> <pct>`    | Move to a specific closed-lift percentage (0-100) |
| `set-tilt <motor> <pct>`        | Set the slat tilt angle on venetian blinds (0-100) |
| `jog-open <motor>`              | Nudge one step open                              |
| `jog-close <motor>`             | Nudge one step closed                            |

Motor control commands are **fire-and-forget** — the CLI exits as soon
as the command is written to the hub. The shade may still be moving
when the CLI returns. Use `motor <motor> position` to poll the current
state.

### Broadcast

| Subcommand | Description                  |
| ---------- | ---------------------------- |
| `open-all` | Open every paired motor      |
| `close-all`| Close every paired motor     |
| `stop-all` | Stop every paired motor      |

### Hub queries

| Subcommand   | Description                                                |
| ------------ | ---------------------------------------------------------- |
| `hub info`   | Full hub info: name, serial, all motors with positions     |
| `hub name`   | Hub friendly name only                                     |
| `hub serial` | Hub serial number only                                     |

### Per-motor queries

| Subcommand                | Description                              |
| ------------------------- | ---------------------------------------- |
| `motor <motor>`           | Show all info for one motor              |
| `motor <motor> name`      | Motor's friendly name                    |
| `motor <motor> position`  | Current closed%, tilt%, and signal       |
| `motor <motor> version`   | Motor type and firmware version          |
| `motor <motor> voltage`   | Battery voltage and signal strength      |

### Dangerous operations (requires `--features dangerous-ops`)

| Subcommand                            | Description                              |
| ------------------------------------- | ---------------------------------------- |
| `dangerous pair`                      | Pair a new motor (hub assigns address)   |
| `dangerous unpair <motor>`            | Unpair motor (requires motor ACK)        |
| `dangerous delete <motor>`            | Delete motor from hub (no ACK needed)    |
| `dangerous factory-reset`             | Wipe all hub configuration               |
| `dangerous set-upper-limit <motor>`   | Set upper limit at current position      |
| `dangerous set-lower-limit <motor>`   | Set lower limit at current position      |
| `dangerous delete-limits <motor>`     | Delete all configured limits             |

## Output format

- Default: human-readable aligned text.
- `--output json`: machine-readable JSON on stdout.

Errors always go to stderr. Exit codes: `0` = success, `1` = runtime
error, `2` = usage error (bad arguments).

## Status and future work

See [`project-tracker.md`](./project-tracker.md) for planned follow-ups.

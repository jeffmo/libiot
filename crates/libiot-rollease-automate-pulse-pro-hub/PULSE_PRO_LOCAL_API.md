# Automate Pulse Pro — Local API Reference

A complete reference for controlling a Rollease Acmeda **Automate Pulse Pro**
(or Pulse 2) hub over the LAN, with **no cloud and no authentication**.

This was reverse-engineered against hub `6217 Shade Hub` (S/N `2016197`) at
`192.168.5.234` by combining:
- The official Rollease Acmeda *Automate Pulse 2 LINQ* integrator PDF (ASCII
  protocol spec)
- The open-source [`aiopulse2`](https://github.com/sillyfrog/aiopulse2) Python
  library (which the community Home Assistant integration is built on)
- Direct probing of the hub

The companion shell script [`blinds.sh`](./blinds.sh) implements the subset
covered in "Everyday operations" below.

---

## 1. Channels at a glance

The hub exposes **two** local endpoints. They carry overlapping but different
data and you pick whichever suits your use case.

| | **ASCII serial** | **WebSocket RPC** |
|---|---|---|
| Port | `1487/tcp` | `443/tcp` (TLS) |
| Framing | `!<addr><cmd><data>;` | JSON text frames over `wss://<hub>/rpc` |
| Auth | none | TLS only; server cert is `CN=AWS IoT Certificate` signed by Amazon (ignore cert validation — it is not minted for the hub's LAN IP) |
| Supported operations | motor control, basic queries, hub name/serial | everything the ASCII channel does + friendly names, rooms, scenes, timers, and real-time push updates |
| Stability | documented by Rollease; stable since Pulse 2 | undocumented — use `aiopulse2` as a reference implementation |
| Dependencies | `nc` | any WebSocket client (`websocat`, Python `websockets`, …) |

**Rule of thumb:** use **ASCII** for shell scripts, cron jobs, Home Assistant
`shell_command`, and anything that needs to be a one-liner. Use **WebSocket RPC**
if you need push updates, room metadata, or scene control.

### 1.1 Prerequisite

The hub must have been paired once via the mobile *Automate Pulse 2* app over
Wi-Fi **before** the local TCP listeners activate. Factory-fresh hubs will not
accept local connections until first-time provisioning is complete. (This is
explicitly noted in Rollease's integrator docs and is not a reverse-engineering
finding.)

---

## 2. ASCII protocol (TCP/1487)

### 2.1 Framing

```
!<address><command>[<data>];
```

| Part | Size | Notes |
|---|---|---|
| `!` | 1 byte | Literal start-of-frame |
| `<address>` | 3 bytes | `[0-9A-Za-z]`. `000` = broadcast/global. Specific motor addresses are assigned by the hub at pairing time (e.g. `4JK`, `MWX`). |
| `<command>` | 1 byte | Non-numeric ASCII character. See table below. |
| `<data>` | 0..N bytes | Command-specific. `?` is the convention for queries. |
| `;` | 1 byte | Literal end-of-frame |

- **Multiple frames** can be concatenated in a single TCP write — the hub
  parses them in order and replies in a stream. This is how you efficiently
  query everything at once.
- The hub is happy with a single long-lived connection; rapid connect/
  disconnect cycles sometimes drop queries. Prefer one connection per burst.
- There is no keepalive / heartbeat. The hub will tolerate idle connections
  for a long time. `aiopulse2` uses the name query (`!000NAME?;`) as a
  synthetic ping.

### 2.2 Commands — motor control (downlink)

All examples use the placeholder motor address `123`.

| Cmd | Example | Purpose |
|---|---|---|
| `o` | `!123o;` | **Open** (lift to 0% — fully up) |
| `c` | `!123c;` | **Close** (lift to 100% — fully down) |
| `s` | `!123s;` | **Stop** in-flight movement |
| `m<NNN>` | `!123m050;` | **Move to position** — `NNN` is the 3-digit closed percentage (`000`..`100`). ⚠ The official PDF documents this as 2 digits; **on-wire it is 3 digits** and `aiopulse2` confirms. |
| `b<NNN>` | `!123b050;` | **Tilt** to 3-digit percentage (for venetian/tilt-capable shades) |
| `oA` | `!123oA;` | **JOG open** — nudge one step up |
| `cA` | `!123cA;` | **JOG close** — nudge one step down |

Broadcast form: replace the 3-char address with `000` to target all motors
paired to the hub (e.g. `!000c;` closes every blind).

### 2.3 Commands — queries (request / response)

| Cmd | Request | Response | Notes |
|---|---|---|---|
| Hub name | `!000NAME?;` | `!000NAME<string>;` | Not in official PDF; from `aiopulse2` |
| Hub serial | `!000SN?;` | `!000SN<string>;` | Not in official PDF; from `aiopulse2` |
| Motor enumeration | `!000v?;` | one `!<addr>v<type><ver>;` per motor, plus the hub's own record | See §2.4 |
| Motor name | `!<addr>NAME?;` | `!<addr>NAME<string>;` | Friendly name set in the mobile app |
| Motor position | `!<addr>r?;` | `!<addr>r<closed3>b<tilt3>,R<sig2>;` | See §2.5. Broadcast form `!000r?;` returns all positions. |
| Motor version | `!<addr>v?;` | `!<addr>v<type><ver>;` | Same format as enumeration response |
| Motor voltage | `!<addr>pVc?;` | `!<addr>pVc<NNNNN>,R<sig2>;` | Battery voltage × 100. `01208` = 12.08 V. |

### 2.4 Motor type codes

From `aiopulse2/const.py`:

| Code | Meaning |
|---|---|
| `A` | AC motor |
| `D` | DC motor |
| `d` | DC motor (lower) |
| `U` | DC motor (U variant) |
| `C` | Curtain motor |
| `B` | **Hub/gateway itself** (filter this out when listing shades) |
| `S` | Socket |
| `L` | Lighting device |

### 2.5 Position-response format (field-verified)

The integrator PDF says `!<addr>r%%b%%;` (2-digit percentages, no trailer).
The **actual on-wire format**, confirmed on firmware running on a Pulse Pro, is:

```
!<addr>r<closed3>b<tilt3>,R<sig2>;
         ^^^^^^^^  ^^^^^^  ^^^^^^
         3 digits  3 dig.  2 hex
```

- `<closed3>` — `000` = fully open, `100` = fully closed
- `<tilt3>` — tilt percentage; can exceed `100` on some motors (observed `180`)
- `<sig2>` — undocumented trailer, **2 uppercase hex chars**, appears on both
  position and voltage responses. Almost certainly an RF link quality /
  signal strength metric for the 433 MHz ARC link between the hub and the
  motor. `aiopulse2` captures it as `signal`.

### 2.6 Pairing / configuration commands (downlink)

Destructive — do **not** run these unless you know what you're doing.

| Cmd | Example | Purpose |
|---|---|---|
| `&` | `!000&;` | Pair a motor — hub auto-assigns a random 3-char address |
| `&<XXX>` | `!000&XYZ;` | Pair a motor with a chosen address `XYZ` |
| `#` | `!123#;` | Unpair motor `123` (requires motor ack) |
| `$` | `!123$;` | Delete motor `123` from hub (no motor ack required) |
| `*` | `!000*;` | **Reset module** — wipes all hub configuration. Factory reset. |
| `pEoH` / `pEcH` | `!123pEoH;` / `!123pEcH;` | Set upper / lower limit on current position |
| `pEoA` / `pEcA` | `!123pEoA;` / `!123pEcA;` | Adjust upper / lower limit |
| `pEaC` | `!123pEaC;` | Delete all limits |
| `pR*` | `!123pR*;` | Return motor to factory mode |

### 2.7 Address-edit feedback

After a successful pair/unpair/rename, the motor echoes:

```
!<addr>A;
```

where `A` stands for "address edit acknowledged".

### 2.8 Error responses

The hub replies with `!<addr>E<xx>;` on errors. `<xx>` is a 2-char code:

| Code | Meaning |
|---|---|
| `bz` | 433 MHz module missed a message while the Wi-Fi module was busy |
| `df` | More than 30 motors paired — hub limit exceeded |
| `np` | No such motor — the address is not in the hub's paired list |
| `nc` | The motor has no upper or lower limits set |
| `mh` | Master Hall sensor abnormal |
| `sh` | Slave Hall sensor abnormal |
| `or` | Motor stalled by obstacle during **up** movement |
| `cr` | Motor stalled by obstacle during **down** movement |
| `pl` | Low-voltage alarm |
| `ph` | High-voltage alarm |
| `nl` | Hub did not get a response from the target motor (RF/range issue) |
| `ec` | Undefined / generic error |

### 2.9 Important quirks

1. **Unsolicited pushes.** The hub sometimes streams state updates for motors
   *other* than the one you queried — treat the connection as a firehose and
   parse every frame, keyed by address.
2. **Minimum 3 chars** — `000` is NOT a wildcard in the general case; it's a
   reserved broadcast address. Responses to a broadcast are *N* separate
   frames, one per motor.
3. **Case-sensitive addresses.** `4jk` ≠ `4JK`. Preserve the case as the hub
   reports it.
4. **No per-command ACKs for motion.** `!123o;`, `!123c;`, `!123s;` don't
   respond when the motor has no limits set, and only respond with the final
   position when it has. Poll `!123r?;` if you need a confirmation.
5. **Room / scene / timer logic is not exposed over ASCII.** The official docs
   are explicit: *"room, scene & timer commands are not supported in ASCII
   commands."* Use the WebSocket channel for those.
6. **Offline motors are silent.** If a motor is off (dead battery, out of RF
   range), it simply won't appear in broadcast responses. This is normal, not
   an error.

---

## 3. Typical workflows

### 3.1 Discover everything in one shot

```sh
{
  printf '!000NAME?;!000SN?;!000v?;!000r?;'
  sleep 3
} | nc -w 4 <hub-ip> 1487
```

Gives you back, in a single stream: hub name, hub serial, every motor's
version record, and every motor's current position. Parse by splitting on
`;` and regex-matching each frame.

### 3.2 Close one blind by friendly name

```sh
#  given MWX -> "Dining Room"
printf '!MWXc;' | nc -w 1 <hub-ip> 1487
```

### 3.3 Move one blind to 50% with feedback

```sh
{
  printf '!MWXm050;'
  sleep 2
  printf '!MWXr?;'
  sleep 2
} | nc -w 4 <hub-ip> 1487
```

### 3.4 Group / scene simulation

The ASCII channel has no scene command. To trigger a "sunset" scene, the
pattern is to concatenate several per-motor moves into a single write:

```sh
printf '!4JKc;!MWXm050;!3YCo;' | nc -w 1 <hub-ip> 1487
```

They execute nearly-simultaneously from the hub's perspective because it
dispatches them to the 433 MHz radio back-to-back.

### 3.5 Graceful error handling

Always parse the response stream looking for `!<addr>E<xx>;` frames. The
most common ones you'll see in day-to-day use are `nl` (RF failure —
motor is offline or out of range) and `or` / `cr` (obstruction).

---

## 4. WebSocket RPC (TCP/443)

The hub exposes a TLS WebSocket at:

```
wss://<hub-ip>/rpc
```

- Server certificate: `CN=AWS IoT Certificate`, issuer `Amazon.com Inc.`,
  validity `2018-02-15` → `2049-12-31`. Unless you happen to trust AWS IoT's
  chain *and* are connecting by that hostname, you must disable cert
  validation. (`aiopulse2` does this with `ssl.CERT_NONE`.)
- No username / password / pre-shared key — connection to the LAN is the
  authentication boundary. Firewall your hub if you care.

The wire protocol is **line-delimited JSON** carrying a custom payload; the
schema is not published by Rollease. The authoritative reference is
[`aiopulse2/devices.py`](https://github.com/sillyfrog/aiopulse2/blob/master/aiopulse2/devices.py)
— look for the `Hub.run()` loop and the `_process_response()` method to see
the message shapes. Key things it exposes that the ASCII channel does not:

- Motor **room** assignments
- Scene definitions
- Timer / schedule configuration
- Real-time position push updates as the motor moves
- Battery percentage (derived from the `Vc` voltage reading against the
  per-model curve — the library codes two curves, one for 8.3V and one for
  12V batteries)

If your use case is "move a shade from a shell script," the ASCII channel is
strictly better. If your use case is "mirror the state of the mobile app,"
the WebSocket is the only game in town and you should use `aiopulse2`
directly rather than re-implementing the JSON schema.

---

## 5. Hub-level context

### 5.1 Cloud architecture

The hub's TLS certificate reveals it is an **AWS IoT Core** device. On the
cloud side, it registers to AWS IoT, talks MQTT/WebSocket with Amazon, and the
mobile app hits the cloud (not your LAN) by default. This has two practical
implications:

- **You can air-gap the hub** by firewall-blocking outbound AWS endpoints
  (`*.iot.*.amazonaws.com`, `*.credentials.iot.*.amazonaws.com`). The local
  ASCII + WebSocket channels will continue to work. You will lose remote
  (away-from-home) app control and any cloud-linked integrations.
- **Every cloud integration is latency-bound by AWS IoT.** Local control via
  TCP/1487 is typically sub-100 ms on a LAN; cloud round-trip is hundreds of
  ms to seconds. For Home Assistant or similar, always prefer local.

### 5.2 Physical layer

The hub talks to motors over a **433 MHz ARC (Automate Radio Communication)**
link. Range is the main failure mode:

- Max 30 motors per hub (error `df` if exceeded).
- RSSI / signal-strength byte on position responses lets you identify
  marginal motors — lower hex = weaker signal (empirically `R58` ≈ strong,
  `R40` ≈ marginal on 2-byte values, but the exact scale is undocumented).
- Rollease sells RF repeaters for large installations.

---

## 6. Reference: observed bytes from `6217 Shade Hub`

Captured via `nc`:

```
→ !000v?;
← !BR1vB10;!4JKvD22;!MWXvD22;!3YCvD22;
    hub:     BR1 type B (hub/gateway), v1.0
    motors:  4JK, MWX, 3YC — all DC, v2.2

→ !000r?;
← !4JKr000b000,R58;!MWXr100b180,R4C;!3YCr000b000,R4C;

→ !000NAME?;
← !000NAME6217 Shade Hub;

→ !000SN?;
← !000SN2016197;

→ !4JKNAME?;!MWXNAME?;!3YCNAME?;
← !4JKNAMEJohn House;!MWXNAMEDining Room;!3YCNAMEKitchen;

→ !4JKpVc?;
← !4JKpVc01208,R58;      (battery = 12.08 V, RF signal 0x58)
```

---

## 7. Further reading

- Rollease Acmeda — [*Automate Pulse 2 LINQ* integrator guide (PDF)](https://rowleycompany.scene7.com/is/content/rowleycompany/rollease-automate-home-integration-linqpdf) — the canonical but incomplete ASCII spec
- [`aiopulse2`](https://github.com/sillyfrog/aiopulse2) — Python async client (reference implementation for both channels)
- [`Automate-Pulse-v2`](https://github.com/sillyfrog/Automate-Pulse-v2) — Home Assistant integration built on `aiopulse2`

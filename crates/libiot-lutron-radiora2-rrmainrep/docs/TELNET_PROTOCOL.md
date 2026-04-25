# Lutron RadioRA 2 RR-MAIN-REP Integration Protocol (Telnet)

This document records the results of reverse-engineering the telnet-based
integration protocol exposed by a Lutron RadioRA 2 RR-MAIN-REP main
repeater (firmware 12.10.0, April 2021). All observations were made
against a live device; no official protocol specification from Lutron was
available at the time of writing.

Community references that informed this work:

- Lutron's own built-in `?HELP` system (the repeater documents itself)
- General knowledge of the Lutron Integration Protocol from community
  integrations (Home Assistant, Hubitat, etc.)

---

## 1. Connection and authentication

### 1.1 Transport

The repeater listens on **TCP port 4323** for integration protocol
connections. The protocol is newline-delimited ASCII text. Commands are
terminated with `\r\n`. Responses are also `\r\n`-terminated.

### 1.2 Login sequence

On connect, the repeater sends:

```
login: 
```

The client sends a username followed by `\r\n`. The repeater then sends:

```
password: 
```

The client sends the password followed by `\r\n`. On success, the
repeater responds with:

```
\r\nGNET> \x00
```

The `GNET> ` prompt (with trailing space and null byte) indicates a
successful login. After login, the connection is ready for commands.

### 1.3 Prompt

After each command response completes, the repeater sends `GNET> ` as a
readiness indicator. However, **the prompt is not a reliable framing
delimiter** --- unsolicited monitoring events can arrive between the end
of a response and the prompt, or interleaved with multi-line responses.

---

## 2. Command format

### 2.1 Command prefixes

| Prefix | Meaning                    | Direction        |
|--------|----------------------------|------------------|
| `#`    | Set / control / action     | Client -> Device |
| `?`    | Query / read               | Client -> Device |
| `~`    | Response / event           | Device -> Client |

### 2.2 General syntax

```
<prefix><COMMAND>,<param1>,<param2>,...\r\n
```

Parameters are comma-separated. Integration IDs are integers. Levels
are floating-point percentages (e.g. `100.00`, `0.00`). Durations use
the format `sec`, `min:sec`, `hr:min:sec`, or `hr:min:sec.hundredths`.

### 2.3 Available commands

The repeater's `?HELP` system lists all supported commands:

| Command           | Description                          |
|-------------------|--------------------------------------|
| `?AREA` / `#AREA` | Area level, scene, and occupancy     |
| `?DEVICE` / `#DEVICE` | Device status, buttons, LEDs     |
| `#EMULATE`        | Emulate device events                |
| `?ETHERNET` / `#ETHERNET` | Network configuration        |
| `?GROUP`          | Occupancy group status               |
| `?HELP`           | Built-in help system                 |
| `?HVAC` / `#HVAC` | HVAC control (if equipped)           |
| `?INTEGRATIONID`  | Integration ID lookup                |
| `?MODE` / `#MODE` | Mode / step control                  |
| `?MONITORING` / `#MONITORING` | Event monitoring setup   |
| `?OUTPUT` / `#OUTPUT` | Output level control and query   |
| `?PARTITIONWALL` / `#PARTITIONWALL` | Partition wall     |
| `#RESET`          | Processor reset                      |
| `?SYSTEM` / `#SYSTEM` | System info and configuration    |
| `?SYSVAR` / `#SYSVAR` | System variables                 |
| `?TIMECLOCK` / `#TIMECLOCK` | Timeclock events and mode  |

Per-command help is available via `?HELP,<command>` and
`?HELP,<command>,<action>`.

### 2.4 Error responses

Errors are returned as `~ERROR,<code>`. Observed error codes:

| Code | Observed meaning                                           |
|------|------------------------------------------------------------|
| 1    | Missing or invalid parameter (e.g. `?SYSTEM` with no arg) |
| 2    | Invalid integration ID / object not found                  |
| 3    | Unsupported query for this device type                     |
| 4    | Invalid component number for this device                   |

---

## 3. Integration ID system

Every addressable object in the RadioRA 2 system has a unique integer
**integration ID**. Objects are typed:

| Type        | Description                                   |
|-------------|-----------------------------------------------|
| `DEVICE`    | Physical device (keypad, sensor, repeater)     |
| `OUTPUT`    | Controllable load (dimmer, switch, shade)      |
| `AREA`      | Logical room / zone grouping                   |
| `TIMECLOCK` | Scheduled event controller                     |

### 3.1 Discovering integration IDs

#### Method 1: Probe with `?INTEGRATIONID`

```
?INTEGRATIONID,3,<id>
```

Returns object info if the ID exists:

```
~INTEGRATIONID,3,<id>,<type>[,<serial>]
```

DEVICE types include a hex serial number. OUTPUT, AREA, and TIMECLOCK
types do not. IDs that don't exist return `~ERROR,2`.

Example responses:

```
~INTEGRATIONID,3,1,DEVICE,0x024C2A4A     (main repeater)
~INTEGRATIONID,3,2,OUTPUT                  (a dimmable output)
~INTEGRATIONID,3,3,AREA                   (a room/area)
~INTEGRATIONID,3,20,TIMECLOCK             (the system timeclock)
```

#### Method 2: XML database export

The repeater serves its full configuration database as XML via:

- **Telnet:** `?SYSTEM,12` (dumps the full XML inline)
- **HTTP:** `GET /DbXmlInfo.xml` on port 4380 (after login at
  `/login?login=<user>&password=<pass>`)

The XML contains the complete area/device/output hierarchy with names,
types, serial numbers, component definitions, button engravings, and
occupancy group assignments. This is the most complete source of device
metadata and is the recommended way to discover the full system
topology.

Key XML structure:

```xml
<Project>
  <ProjectName ProjectName="..." />
  <Areas>
    <Area Name="Root" IntegrationID="0">
      <Areas>
        <Area Name="Living Room" IntegrationID="13">
          <Outputs>
            <Output Name="Overhead Lights" IntegrationID="22"
                    OutputType="INC" Wattage="0" />
          </Outputs>
          <DeviceGroups>
            <DeviceGroup>
              <Devices>
                <Device Name="..." IntegrationID="24"
                        DeviceType="HYBRID_SEETOUCH_KEYPAD"
                        SerialNumber="51605520">
                  <Components>
                    <Component ComponentNumber="1" ComponentType="BUTTON">
                      <Button Engraving="Kitchen" />
                    </Component>
                    <Component ComponentNumber="81" ComponentType="LED" />
                  </Components>
                </Device>
              </Devices>
            </DeviceGroup>
          </DeviceGroups>
        </Area>
      </Areas>
    </Area>
  </Areas>
</Project>
```

### 3.2 DB export timestamp

```
?SYSTEM,10
~SYSTEM,05/05/2024,12:14:40
```

Returns the date and time the database was last exported / programmed.

---

## 4. Output control (`#OUTPUT` / `?OUTPUT`)

Outputs are the primary interface for controlling lights, dimmers, and
shades. Each output has an integration ID.

### 4.1 Query output level

```
?OUTPUT,<id>,1
~OUTPUT,<id>,1,<level>
```

Level is a float from `0.00` (off) to `100.00` (full on).

### 4.2 Query other output states

| Action | Description                      | Response format                  |
|--------|----------------------------------|----------------------------------|
| 1      | Current level/position           | `~OUTPUT,<id>,1,<level>`         |
| 5      | Flash frequency                  | `~OUTPUT,<id>,5,<freq_hz>`       |
| 6      | Pulse time                       | `~OUTPUT,<id>,6,<time_sec>`      |
| 7      | Daylighting state                | `~OUTPUT,<id>,7`                 |
| 29     | Last change source (Quantum)     | `~OUTPUT,<id>,29,<feature_code>` |

Feature codes for action 29 (last change source):

| Code | Source                        |
|------|-------------------------------|
| 0    | Unknown                       |
| 6    | Integration (telnet/API)      |
| 7    | LEAP                          |
| 8    | Keypad                        |
| 10   | Occupancy (occupied)          |
| 11   | Occupancy (unoccupied)        |
| 15   | Sequence                      |
| 16   | Timeclock                     |
| 24   | Local device control          |

(Full list available via `?HELP,?OUTPUT`)

### 4.3 Set output level (dimming)

```
#OUTPUT,<id>,1,<level>[,<fade_time>[,<delay>]]
```

- `level`: 0.00 to 100.00 (percentage)
- `fade_time`: optional, default instant. Format: `sec`,
  `min:sec`, `hr:min:sec`, or `hr:min:sec.hundredths`
- `delay`: optional, seconds before starting the fade

Examples:

```
#OUTPUT,22,1,100            Turn on to 100%
#OUTPUT,22,1,0              Turn off
#OUTPUT,22,1,50,3           Fade to 50% over 3 seconds
#OUTPUT,22,1,75,1:30        Fade to 75% over 1 minute 30 seconds
#OUTPUT,22,1,0,5,2          Fade to 0% over 5 seconds after 2s delay
```

**For NON_DIM outputs:** The level is effectively boolean: `0.00` = off,
any non-zero value = on. The repeater accepts the command but the
physical device only supports on/off.

### 4.4 Other output actions

| Action | Command                                       | Description              |
|--------|-----------------------------------------------|--------------------------|
| 1      | `#OUTPUT,<id>,1,<level>[,<fade>[,<delay>]]`   | Set level with fade      |
| 2      | `#OUTPUT,<id>,2`                              | Start raising            |
| 3      | `#OUTPUT,<id>,3`                              | Start lowering           |
| 4      | `#OUTPUT,<id>,4`                              | Stop raise/lower         |
| 5      | `#OUTPUT,<id>,5,<fade>[,<delay>]`             | Flash                    |
| 6      | `#OUTPUT,<id>,6,<pulse_time>`                 | Pulse (0.1-10.0 sec)     |
| 9      | `#OUTPUT,<id>,9,<tilt_level>[,<delay>]`       | Shade tilt               |
| 10     | `#OUTPUT,<id>,10,<lift>,<tilt>[,<delay>]`     | Shade lift + tilt        |
| 17     | `#OUTPUT,<id>,17,<color_idx>[,<fade>[,<delay>]]` | DMX color/level      |
| 18     | `#OUTPUT,<id>,18`                             | Jog raise                |
| 19     | `#OUTPUT,<id>,19`                             | Jog lower                |

### 4.5 Output types (from XML database)

| OutputType | Description                                  |
|------------|----------------------------------------------|
| `INC`      | Incandescent / resistive (fully dimmable)    |
| `ELV`      | Electronic Low Voltage (dimmable)            |
| `NON_DIM`  | Non-dimmable (on/off only)                   |
| `MLV`      | Magnetic Low Voltage                         |
| `SHADE`    | Shade / blind                                |

---

## 5. Device queries and control (`?DEVICE` / `#DEVICE`)

Devices are physical hardware: keypads, motion sensors, the repeater
itself. Each device has components (buttons, LEDs, CCIs).

### 5.1 Device types (from XML database)

| DeviceType                 | Description                            |
|----------------------------|----------------------------------------|
| `MOTION_SENSOR`            | Occupancy / vacancy sensor             |
| `HYBRID_SEETOUCH_KEYPAD`  | Wall keypad with buttons and LEDs      |
| (main repeater)            | The RR-MAIN-REP itself (ID 1)         |

### 5.2 Component types

| ComponentType | Typical numbers | Description                     |
|---------------|----------------|---------------------------------|
| `BUTTON`      | 1-19           | Physical button on a keypad     |
| `LED`         | 81-85          | LED indicator on a keypad       |
| `CCI`         | 2              | Contact closure input (sensor)  |

### 5.3 Query device enable status

```
?DEVICE,<id>,0,1
~DEVICE,<id>,00000,1,<state>
```

State: 1 = enabled, 2 = disabled.

### 5.4 Query LED state

```
?DEVICE,<id>,<component>,9
~DEVICE,<id>,<component>,9,<led_state>
```

| LED state | Meaning    |
|-----------|------------|
| 0         | Off        |
| 1         | On         |
| 2         | Flash slow |
| 3         | Flash fast |
| 255       | Unknown    |

### 5.5 Query current scene

```
?DEVICE,<id>,0,7
~DEVICE,<id>,00000,7,<scene_number>
```

Scene number: 0-32, or 255 for none/unknown.

### 5.6 Query battery status

```
?DEVICE,<id>,0,22
~DEVICE,<id>,0,22,<serial>,<power_source>,<status>,<timestamp>
```

| Power source | Meaning                  |
|--------------|--------------------------|
| 0            | Unknown                  |
| 1            | Battery powered          |
| 2            | Externally powered       |

| Status | Meaning               |
|--------|------------------------|
| 0      | Not battery powered    |
| 1      | Unknown                |
| 2      | Good                   |
| 3      | Low                    |
| 4      | MIA (missing in action)|
| 5      | Not activated          |

### 5.7 Control actions

| Action | Command                                    | Description       |
|--------|--------------------------------------------|-------------------|
| 1      | `#DEVICE,<id>,0,1`                         | Enable device     |
| 2      | `#DEVICE,<id>,0,2`                         | Disable device    |
| 3      | `#DEVICE,<id>,<comp>,3`                    | Button press      |
| 4      | `#DEVICE,<id>,<comp>,4`                    | Button release    |
| 5      | `#DEVICE,<id>,<comp>,5`                    | Button hold       |
| 6      | `#DEVICE,<id>,<comp>,6`                    | Multi-tap         |
| 7      | `#DEVICE,<id>,0,7,<scene>`                 | Select scene      |
| 9      | `#DEVICE,<id>,<comp>,9,<state>`            | Set LED state     |
| 32     | `#DEVICE,<id>,<comp>,32`                   | Release from hold |

---

## 6. Area queries (`?AREA` / `#AREA`)

Areas are logical groupings of devices and outputs (rooms, zones).

### 6.1 Query area level

```
?AREA,<id>,1
~AREA,<id>,1
```

When queried, the repeater responds with the area acknowledgement and
then separately reports the level of each output in the area:

```
~AREA,<id>,1
~OUTPUT,<output_id_1>,1,<level>
~OUTPUT,<output_id_2>,1,<level>
...
```

This is a convenient way to get all output levels for a room in one
query. Areas with no outputs return only the area acknowledgement.

### 6.2 Query area scene

```
?AREA,<id>,6
~AREA,<id>,6,<scene>
```

Scene is 0-32, or `XX` when no scene is active.

### 6.3 Query area occupancy

```
?AREA,<id>,8
~AREA,<id>,8,<state>
```

| State | Meaning     |
|-------|-------------|
| 1     | Unknown     |
| 2     | Inactive    |
| 3     | Occupied    |
| 4     | Unoccupied  |

### 6.4 Other area queries

| State | Description                     |
|-------|---------------------------------|
| 1     | Level/position                  |
| 6     | Current scene                   |
| 7     | Daylighting state               |
| 8     | Occupancy state                 |
| 9     | Occupancy active state          |
| 10    | Area power used                 |
| 12    | Occupied level or scene         |
| 13    | Unoccupied level or scene       |
| 22    | Occupancy low light setting     |
| 29    | Max power available             |
| 30    | Power savings                   |

### 6.5 Area control

```
#AREA,<id>,1,<level>[,<fade>[,<delay>]]   Set all outputs to level
#AREA,<id>,2                               Raise all
#AREA,<id>,3                               Lower all
#AREA,<id>,4                               Stop raise/lower
#AREA,<id>,6,<scene>                       Activate scene
```

---

## 7. Occupancy groups (`?GROUP`)

Occupancy groups aggregate sensor state across areas.

```
?GROUP,<id>,3
~GROUP,<id>,3,<state>
```

| State | Meaning     |
|-------|-------------|
| 2     | Inactive    |
| 3     | Occupied    |
| 4     | Unoccupied  |
| 255   | Unknown     |

Note: Group IDs appear to correspond to area integration IDs in this
installation, not the OccupancyGroupAssignedToID from the XML database.

---

## 8. Real-time monitoring (`#MONITORING` / `?MONITORING`)

This is a critical feature for building a streaming state-tracking API.
Monitoring allows the client to subscribe to real-time event
notifications for specific categories.

### 8.1 Enable/disable monitoring

```
#MONITORING,<type>,1      Enable monitoring type
#MONITORING,<type>,2      Disable monitoring type
#MONITORING,255,1         Enable ALL monitoring types (except 11, 12)
#MONITORING,255,2         Disable ALL monitoring types (except 11, 12)
```

**Important:** The `255` (all) command does **not** affect types 11
(Reply State) and 12 (Prompt State). Those must be enabled/disabled
individually.

### 8.2 Query monitoring status

```
?MONITORING,<type>
~MONITORING,<type>,<state>       (1=enabled, 2=disabled)
```

### 8.3 Monitoring types

| Type | Name                              | Events produced                          |
|------|-----------------------------------|------------------------------------------|
| 1    | Diagnostic Monitoring             | Internal errors, DNS failures, etc.      |
| 2    | Event Monitoring                  | General system events                    |
| 3    | Button Monitoring                 | Keypad button presses/releases           |
| 4    | LED Monitoring                    | Keypad LED state changes                 |
| 5    | Zone Monitoring                   | Output level changes                     |
| 6    | Occupancy Monitoring              | Occupancy sensor triggers                |
| 7    | Photosensor Monitoring            | Light level sensor readings              |
| 8    | Scene Monitoring                  | Scene activations                        |
| 9    | Time Clock Monitoring             | Timeclock events                         |
| 10   | System Variable Monitoring        | System variable changes                  |
| 11   | Reply State                       | Echo command responses (not in `255`)    |
| 12   | Prompt State                      | Echo `GNET>` prompts (not in `255`)      |
| 14   | Device State Monitoring           | Device enable/disable changes            |
| 15   | Address Monitoring                | Address assignments                      |
| 16   | Sequence Monitoring               | Sequence executions                      |
| 17   | HVAC Monitoring                   | HVAC state changes                       |
| 18   | Mode Monitoring                   | Mode changes                             |
| 19   | Preset Transfer Monitoring        | Preset transfers                         |
| 20   | L1 Runtime Property Monitoring    | L1 property updates                      |
| 21   | L2 Runtime Property Monitoring    | L2 property updates                      |
| 23   | Shade Group Monitoring            | Shade group changes                      |
| 24   | Partition Wall Monitoring         | Partition wall changes                   |
| 25   | System Monitoring                 | System-level events                      |
| 26   | Hyperion Sensor Grouping          | Daylight sensor grouping                 |

Note: Type 13 and 22 are not listed by the repeater.

### 8.4 Observed monitoring event formats

When monitoring is enabled, the repeater sends unsolicited events on the
same TCP connection. These events use the standard `~` response prefix
and `\r\n` line termination. Events arrive asynchronously --- they can
interleave with command responses.

#### Output level change (Zone Monitoring, type 5)

```
~OUTPUT,<id>,1,<level>
```

Sent whenever an output's level changes, whether from a keypad press,
occupancy trigger, timeclock event, or integration command.

Additional output event fields observed:

```
~OUTPUT,<id>,29,<feature_code>      Source of level change
~OUTPUT,<id>,30,1,<level>           (observed, exact meaning TBD)
```

#### Device events (Button/LED/Device State Monitoring, types 3/4/14)

Button press/release:

```
~DEVICE,<id>,<component>,3         Button pressed
~DEVICE,<id>,<component>,4         Button released
```

LED state change:

```
~DEVICE,<id>,<component>,9,<state>
```

Where `<state>` is 0 (off), 1 (on), 2 (flash slow), 3 (flash fast).

Motion sensor CCI events:

```
~DEVICE,<id>,<component>,3         CCI closed (motion detected)
~DEVICE,<id>,<component>,4         CCI opened (motion cleared)
```

#### Occupancy group events (Occupancy Monitoring, type 6)

```
~GROUP,<id>,3,<state>
```

Where `<state>` is 3 (occupied) or 4 (unoccupied).

#### Diagnostic events (type 1)

```
Diag Error On Line <N> of <file>
<error description>
```

These are unformatted diagnostic messages (no `~` prefix) that appear
inline. Common examples:

```
DNS socket open on local port 9
Diag Error On Line 413 of dns_private.cpp
Didn't find A record response in DNS response
```

#### System events

```
FLASH WRITE
Export Date and Time Unchanged
Port is not registered
```

These are unformatted system messages that can appear at any time.

### 8.5 Event stream architecture considerations

For building a streaming API, key observations:

1. **Single connection, multiplexed:** All events and responses share
   one TCP connection. The client must demultiplex responses to its own
   queries from unsolicited monitoring events.

2. **No request-response correlation:** There is no request ID or
   sequence number. The client must match responses by command type and
   integration ID.

3. **Interleaving:** Monitoring events can arrive mid-response. For
   example, while reading output levels for an area query, motion sensor
   events or other output changes can appear inline.

4. **Line-based framing:** Each event or response is a single line
   terminated by `\r\n`. Some diagnostic messages span multiple lines.

5. **Prompt as delimiter:** The `GNET> ` prompt appears after command
   responses complete, but is unreliable as a response boundary due to
   event interleaving.

6. **Recommended monitoring types for a home automation client:**
   - Type 5 (Zone) --- output level changes
   - Type 3 (Button) --- keypad button events
   - Type 4 (LED) --- keypad LED state
   - Type 6 (Occupancy) --- motion sensor state
   - Type 8 (Scene) --- scene activations

---

## 9. System queries (`?SYSTEM`)

| Action | Command         | Response format                              |
|--------|-----------------|----------------------------------------------|
| 1      | `?SYSTEM,1`     | `~SYSTEM,1,HH:MM:SS` (current time)         |
| 2      | `?SYSTEM,2`     | `~SYSTEM,2,MM/DD/YYYY` (current date)        |
| 4      | `?SYSTEM,4`     | `~SYSTEM,4,<lat>,<long>`                     |
| 5      | `?SYSTEM,5`     | `~SYSTEM,5,<utc_offset>`                     |
| 6      | `?SYSTEM,6`     | `~SYSTEM,6,HH:MM:SS` (sunset)               |
| 7      | `?SYSTEM,7`     | `~SYSTEM,7,HH:MM:SS` (sunrise)              |
| 8      | `?SYSTEM,8`     | Free-form: `OS Firmware Revision = <ver>`    |
| 10     | `?SYSTEM,10`    | `~SYSTEM,MM/DD/YYYY,HH:MM:SS` (DB export)   |
| 12     | `?SYSTEM,12`    | Full XML database (inline, same as HTTP)     |
| 14     | `?SYSTEM,14`    | `System Statuses:` + status lines            |
| 22     | `?SYSTEM,22`    | Battery status for all devices               |

---

## 10. Ethernet configuration (`?ETHERNET`)

| Selection | Command          | Response                                |
|-----------|------------------|-----------------------------------------|
| 0         | `?ETHERNET,0`    | `~ETHERNET,0,<ip>` (IP address)         |
| 1         | `?ETHERNET,1`    | `~ETHERNET,1,<gateway>`                 |
| 2         | `?ETHERNET,2`    | `~ETHERNET,2,<subnet>`                  |
| 4         | `?ETHERNET,4`    | `~ETHERNET,4,<dhcp>` (1=on)             |
| 5         | `?ETHERNET,5`    | `~ETHERNET,5,<multicast_addr>`          |

Note: Selection 3 (login info) is documented but returns ERROR 3.

---

## 11. Timeclock (`?TIMECLOCK`)

```
?TIMECLOCK,<id>,1                   Current mode
~TIMECLOCK,<id>,1,<mode>

?TIMECLOCK,<id>,2                   Sunrise time
~TIMECLOCK,<id>,2,HH:MM

?TIMECLOCK,<id>,3                   Sunset time
~TIMECLOCK,<id>,3,HH:MM

?TIMECLOCK,<id>,4                   Day's schedule
~TIMECLOCK, Event ID = 0x<hex>, Event Time = HH:MM, ...
```

---

## 12. HTTP API (port 4380)

The repeater also exposes a minimal HTTP interface on port 4380.

### 12.1 Authentication

```
GET /login?login=<user>&password=<pass>
```

Returns an HTML page with links to available endpoints.

### 12.2 Available endpoints

| Endpoint          | Description                                     |
|-------------------|-------------------------------------------------|
| `/DeviceIP`       | Device IP information                           |
| `/DbXmlInfo.xml`  | Full XML database export (same as `?SYSTEM,12`) |

The HTTP API is limited and slow compared to the telnet interface. The
telnet interface is strongly preferred for integration.

---

## 13. Observed device inventory

This section documents the specific devices found on the test system
for reference during crate development.

### 13.1 Integration ID map

| ID | Type      | Name / Description                                        |
|----|-----------|-----------------------------------------------------------|
| 1  | DEVICE    | Main repeater (RR-MAIN-REP, serial 0x024C2A4A)           |
| 2  | OUTPUT    | Master Bedroom - Overhead Lights (INC, dimmable)          |
| 3  | AREA      | Den                                                       |
| 4  | DEVICE    | Downstairs Entryway - Motion Sensor (serial 0x025F8145)   |
| 5  | AREA      | Downstairs Entryway                                       |
| 6  | AREA      | Project Room                                              |
| 7  | AREA      | Upstairs Entryway                                         |
| 8  | AREA      | Stairwell                                                 |
| 9  | AREA      | Garage                                                    |
| 10 | AREA      | Guest Bath                                                |
| 11 | AREA      | Guest Bedroom                                             |
| 12 | DEVICE    | Stairwell - Motion Sensor (serial 0x00CDDB31)             |
| 13 | AREA      | Living Room                                               |
| 14 | AREA      | Master Bath                                               |
| 15 | AREA      | Master Bedroom                                            |
| 16 | AREA      | Office                                                    |
| 17 | --        | (unassigned)                                              |
| 18 | AREA      | Upstairs Hall                                             |
| 19 | --        | GREEN_MODE (from XML, returns ERROR via telnet)           |
| 20 | TIMECLOCK | System timeclock                                          |
| 21 | OUTPUT    | Living Room - Floor Lamp (NON_DIM, on/off only)           |
| 22 | OUTPUT    | Living Room - Overhead Lights (INC, dimmable)             |
| 23 | OUTPUT    | Upstairs Entryway - Dining Cans (INC, dimmable)           |
| 24 | DEVICE    | Upstairs Entry Keypad (HYBRID_SEETOUCH_KEYPAD)            |
| 25 | OUTPUT    | Upstairs Hall - Kitchen Cans (INC, dimmable)              |
| 26 | DEVICE    | Upstairs Hallway Keypad (HYBRID_SEETOUCH_KEYPAD)          |
| 27 | OUTPUT    | Upstairs Entryway - Entryway Pendant (ELV, dimmable)      |
| 28 | OUTPUT    | Upstairs Entryway - Dining Pendant (ELV, dimmable)        |
| 29 | OUTPUT    | Downstairs Entryway - Ceiling Cans (INC, dimmable)        |

### 13.2 Keypad component layout

Both seeTouch keypads (IDs 24, 26) share the same layout:

| Component | Type   | Description                                    |
|-----------|--------|------------------------------------------------|
| 1-5       | BUTTON | Main scene buttons (with engravings)           |
| 18-19     | BUTTON | Raise/lower buttons (no engraving)             |
| 81-85     | LED    | LED indicators (one per main button)           |

Button engravings vary per keypad and represent the scene or output
that button controls.

---

## 14. Protocol quirks and implementation notes

### 14.1 Response parsing challenges

1. **No framing delimiter:** Unlike some protocols, there is no
   end-of-response marker. The `GNET>` prompt is suggestive but not
   reliable.

2. **Mixed structured and unstructured output:** Most responses use the
   `~<COMMAND>,<params>` format, but diagnostics (DNS errors, FLASH
   WRITE) are raw text with no prefix.

3. **Null bytes:** The login response includes a trailing `\x00` after
   the first `GNET>` prompt. This may need to be stripped.

4. **`\r\n` vs `\r\r\n`:** Some multi-line help responses use `\r\r\n`
   instead of `\r\n`. The parser should normalize line endings.

5. **Unsolicited events:** With monitoring enabled, events can arrive
   at any time, including during command responses. The transport layer
   must handle demultiplexing.

### 14.2 Connection lifecycle

- The repeater supports multiple concurrent telnet connections.
- Connections are long-lived; no keepalive mechanism was observed.
- Monitoring subscriptions are per-connection and reset on disconnect.
- The `GNET>` prompt is sent after each command but should not be
  relied upon for framing.

### 14.3 Rate limiting

No explicit rate limiting was observed. The repeater responds promptly
to sequential commands with minimal delay between responses (sub-100ms
for most queries).

### 14.4 Command batching

Multiple commands can be sent in rapid succession. The repeater queues
and processes them sequentially, sending responses in order. However,
monitoring events can interleave between queued command responses.

### 14.5 NON_DIM output behavior

For `NON_DIM` outputs (e.g., Floor Lamp, ID 21), the `#OUTPUT` set
level command is accepted but the physical device only supports on/off.
`?OUTPUT,21,1` reports either `0.00` or `100.00`.

---

## 15. Crate API design implications

Based on the protocol analysis, the crate should support:

### 15.1 One-shot queries (request/response)

- Query single output level: `?OUTPUT,<id>,1`
- Query all outputs in an area: `?AREA,<id>,1`
- Query device status (LEDs, enable state, battery)
- Query system info (time, date, firmware, sunrise/sunset)
- Query occupancy state per area or group
- Fetch full device database (XML export)

### 15.2 Control commands (fire-and-forget with confirmation)

- Set output level with optional fade and delay
- Raise / lower / stop outputs
- Activate scenes (via area or device)
- Press/release keypad buttons (via device)

### 15.3 Streaming event subscription

- Subscribe to monitoring types (zone, button, LED, occupancy, etc.)
- Receive typed event stream (output changes, button presses, motion
  sensor triggers, LED state changes)
- Demultiplex events from command responses on a single connection

### 15.4 Device discovery

- Integration ID probe (`?INTEGRATIONID,3,<id>`)
- XML database fetch and parse (recommended for full topology)

### 15.5 Transport layer considerations

- TCP connection on port 4323
- Login sequence before commands
- Accumulator-based line parser (handle partial reads, `\r\n`
  termination, null bytes, diagnostic noise)
- Monitoring event demux from command responses
- Consider a dedicated background reader task that:
  - Feeds monitoring events to a broadcast channel
  - Routes command responses to waiting request futures

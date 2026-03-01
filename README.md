# HC Smarthome Specification

This document defines a semantic layer for smart home devices built on the
[Homie 5 MQTT convention](https://homieiot.github.io/). It provides a set of
reusable **capabilities** (composable building blocks for device features) and
**device classes** (physical device identity) that enable interoperable device
descriptions, user interfaces, and automation engines.

## Homie 5 Mapping

The HC Smarthome spec layers on top of two existing Homie 5 fields:

| Concept      | Homie 5 Level | Field  | Naming Pattern              |
| ------------ | ------------- | ------ | --------------------------- |
| Capability   | Node          | `type` | `hc-smarthome/v2/cap/<name>`  |
| Device class | Device        | `type` | `hc-smarthome/v2/dc/<name>`   |
| Extension    | Node          | `type` | `hc-smarthome/v2/ext/<name>`  |

**Namespace:** `hc-smarthome/v2`

A Homie device composes one or more capability nodes. Each node's `type` field
identifies which capability it implements. The device's own `type` field
identifies its device class. Controllers and UIs use these strings to select
appropriate controls, icons, and interaction behavior.

## Concepts

### Capabilities

A **capability** is a single, reusable feature that a device can expose.
Each capability maps to exactly one Homie node with a well-known set of
properties, datatypes, and constraints.

Capabilities are independent by default. When composed on the same device,
**interaction rules** (defined per device class) specify how capabilities
affect each other.

Examples:
- A wall plug exposes a single `switch` capability.
- A dimmable bulb exposes `switch` + `level`.
- A full-color bulb exposes `switch` + `level` + `color`.
- A wall thermostat exposes `thermostat` + `climate`.

### Device Classes

A **device class** identifies what the physical device *is*. It is advisory:
it helps UIs render appropriate icons and controls, and it defines interaction
rules between capabilities. A device class does **not** constrain which
capabilities a device may expose -- any device can have any combination of
capabilities.

### RFC 2119 Language

The key words "MUST", "MUST NOT", "SHOULD", "SHOULD NOT", and "MAY" in this
document are to be interpreted as described in
[RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

---

## Capabilities at a Glance

| Capability  | Default ID    | Type String                       | Category       | Description                                   |
| ----------- | ------------- | --------------------------------- | -------------- | --------------------------------------------- |
| Switch      | `switch`      | `hc-smarthome/v2/cap/switch`      | Actuator       | On/off control with toggle action             |
| Level       | `level`       | `hc-smarthome/v2/cap/level`       | Actuator       | Percentage level control (0-100%)             |
| Color       | `color`       | `hc-smarthome/v2/cap/color`       | Actuator       | Color and color-temperature control           |
| Scene       | `scene`       | `hc-smarthome/v2/cap/scene`       | Actuator       | Recall named scenes                           |
| Shutter     | `shutter`     | `hc-smarthome/v2/cap/shutter`     | Actuator       | Blind/shutter position and direction control  |
| Thermostat  | `thermostat`  | `hc-smarthome/v2/cap/thermostat`  | Actuator       | Heating/cooling setpoint and mode control     |
| Lock        | `lock`        | `hc-smarthome/v2/cap/lock`        | Actuator       | Lock/unlock control                           |
| Valve       | `valve`       | `hc-smarthome/v2/cap/valve`       | Actuator       | Binary valve open/close control               |
| Climate     | `climate`     | `hc-smarthome/v2/cap/climate`     | Sensor         | Temperature, humidity, pressure sensing       |
| Motion      | `motion`      | `hc-smarthome/v2/cap/motion`      | Sensor         | Motion detection with optional light level    |
| Vibration   | `vibration`   | `hc-smarthome/v2/cap/vibration`   | Sensor         | Vibration detection with optional strength    |
| Contact     | `contact`     | `hc-smarthome/v2/cap/contact`     | Sensor         | Open/close contact sensing                    |
| Water       | `water`       | `hc-smarthome/v2/cap/water`       | Sensor         | Water leak detection                          |
| Tilt        | `tilt`        | `hc-smarthome/v2/cap/tilt`        | Sensor         | Binary tilt detection                         |
| Orientation | `orientation` | `hc-smarthome/v2/cap/orientation` | Sensor         | 3-axis orientation and tilt angle             |
| Button      | `button`      | `hc-smarthome/v2/cap/button`      | Infrastructure | Physical button press events                  |
| Powermeter  | `powermeter`  | `hc-smarthome/v2/cap/powermeter`  | Infrastructure | Electrical power metering                     |
| Mediaplayer | `mediaplayer` | `hc-smarthome/v2/cap/mediaplayer` | Media          | Playback transport control and modes          |
| Media Info  | `media-info`  | `hc-smarthome/v2/cap/media-info`  | Media          | Now-playing metadata and progress             |
| Volume      | `volume`      | `hc-smarthome/v2/cap/volume`      | Media          | Audio volume level and mute control           |
| Battery     | `battery`     | `hc-smarthome/v2/cap/battery`     | Infrastructure | Battery level and voltage readings            |
| Link        | `link`        | `hc-smarthome/v2/cap/link`        | Infrastructure | Signal strength, link quality, last-seen      |

---

## Device Classes

### Overview

| Device Class   | Type String                         | Required     | Optional                               | Description                      |
| -------------- | ----------------------------------- | ------------ | -------------------------------------- | -------------------------------- |
| Light          | `hc-smarthome/v2/dc/light`          | `switch`     | `level`, `color`, `scene`              | Light fixture or smart bulb      |
| Outlet         | `hc-smarthome/v2/dc/outlet`         | `switch`     | `powermeter`                           | Power outlet or smart plug       |
| Thermostat     | `hc-smarthome/v2/dc/thermostat`     | `thermostat` | `climate`                              | Wall or standalone thermostat    |
| Radiator Valve | `hc-smarthome/v2/dc/radiator-valve` | `thermostat` | `climate`                              | Thermostatic radiator valve      |
| Climate Sensor | `hc-smarthome/v2/dc/climate-sensor` | `climate`    | --                                     | Temperature/humidity sensor      |
| Motion Sensor  | `hc-smarthome/v2/dc/motion-sensor`  | `motion`     | `climate`                              | Motion/occupancy sensor          |
| Contact Sensor | `hc-smarthome/v2/dc/contact-sensor` | `contact`    | --                                     | Door/window contact sensor       |
| Water Sensor   | `hc-smarthome/v2/dc/water-sensor`   | `water`      | --                                     | Water leak/flood sensor          |
| Lock           | `hc-smarthome/v2/dc/lock`           | `lock`       | --                                     | Door lock or safe                |
| Shutter        | `hc-smarthome/v2/dc/shutter`        | `shutter`    | --                                     | Window blind, shade, or shutter  |
| Fan            | `hc-smarthome/v2/dc/fan`            | `switch`     | `level`                                | Ceiling or standing fan          |
| Valve          | `hc-smarthome/v2/dc/valve`          | `valve`      | --                                     | Water or gas shutoff valve       |
| Button         | `hc-smarthome/v2/dc/button`         | `button`     | --                                     | Physical push-button or remote   |
| Siren          | `hc-smarthome/v2/dc/siren`          | `switch`     | --                                     | Alarm siren                      |
| Powermeter     | `hc-smarthome/v2/dc/powermeter`     | `powermeter` | --                                     | Standalone power meter or clamp  |
| Mediaplayer    | `hc-smarthome/v2/dc/mediaplayer`    | `mediaplayer`| `media-info`, `volume`                 | Media player, smart speaker, TV  |

Any device class MAY additionally expose `battery` and/or `link` capability
nodes. These are optional for all device classes and provide device health
readings (battery level/voltage, signal strength, link quality, last-seen
timestamp). Binary health conditions (low-battery, unreachable) are handled
by the [Alerts](#alerts) system instead.

### Interaction Rules

Most device classes have no cross-capability interaction rules -- each
capability on the device operates independently. The exceptions are documented
below.

#### Light and Fan: Switch + Level Coupling

When a device of class `light` or `fan` exposes both `switch` and `level`
capabilities, the following rules apply:

- When `level/value` is set to **0**, `switch/state` SHOULD be set to `false`.
- When `level/value` is set to a value **> 0**, `switch/state` SHOULD be set
  to `true`.
- When `switch/state` is set to **`false`**, `level/value` SHOULD be set
  to `0`.
- When `switch/state` is set to **`true`**, `level/value` SHOULD be restored
  to the **last non-zero value**. If no prior non-zero value exists, it SHOULD
  default to `100`.

The bridge or adapter implementing the device is responsible for enforcing
these rules.

`color` and `scene` capabilities are **not** affected by the switch/level
coupling -- they operate independently.

---

## How to Read Property Tables

| Column       | Meaning                                                                 |
| ------------ | ----------------------------------------------------------------------- |
| **Property** | Human-readable name shown in the Homie device description               |
| **ID**       | Homie property ID used in MQTT topics                                   |
| **Datatype** | Homie datatype (`Boolean`, `Integer`, `Float`, `Enum`, `Color`, `Datetime`) |
| **Unit**     | Measurement unit (`--` = none)                                          |
| **Format**   | Homie format constraint (`--` = none)                                   |
| **Settable** | Whether the property accepts `/set` commands                            |
| **Retained** | Whether the MQTT message uses the retain flag                           |
| **Optional** | Whether the property may be absent from the node                        |

---

## Capability Reference

### Actuator Capabilities

#### Switch

**ID:** `switch` | **Type:** `hc-smarthome/v2/cap/switch`

On/off control with a toggle action. Used for binary actuators like lights,
relays, or power outlets.

| Property     | ID       | Datatype | Unit | Format                       | Settable | Retained | Optional | Description              |
| ------------ | -------- | -------- | ---- | ---------------------------- | -------- | -------- | -------- | ------------------------ |
| On/Off state | `state`  | Boolean  | --   | `false="off"`, `true="on"`   | yes      | yes      | no       | Current on/off state     |
| Change state | `action` | Enum     | --   | `toggle`                     | yes      | no       | no       | Trigger a toggle action  |

---

#### Level

**ID:** `level` | **Type:** `hc-smarthome/v2/cap/level`

Percentage level control (0-100%) with optional step actions. Used for
brightness, fan speed, volume, or any percentage-based control.

| Property   | ID       | Datatype | Unit | Format                   | Settable | Retained | Optional | Description             |
| ---------- | -------- | -------- | ---- | ------------------------ | -------- | -------- | -------- | ----------------------- |
| Level      | `value`  | Integer  | `%`  | `0:100`                  | yes      | yes      | no       | Current level (0-100%)  |
| Step level | `action` | Enum     | --   | `step-up`, `step-down`   | yes      | no       | yes      | Relative step change    |

---

#### Color

**ID:** `color` | **Type:** `hc-smarthome/v2/cap/color`

Color control supporting color values (RGB/HSV/XYZ) and color temperature
in mireds.

| Property          | ID                 | Datatype | Unit | Format                    | Settable | Retained | Optional | Description                 |
| ----------------- | ------------------ | -------- | ---- | ------------------------- | -------- | -------- | -------- | --------------------------- |
| Color             | `color`            | Color    | --   | Color formats (e.g. `rgb`) | yes      | yes      | no       | Current color value         |
| Color temperature | `color-temperature` | Integer  | --   | `{ctmin}:{ctmax}`          | yes      | yes      | no       | Color temperature in mireds |

Default color-temperature range: `153:555` (approx. 1800K-6500K).

---

#### Scene

**ID:** `scene` | **Type:** `hc-smarthome/v2/cap/scene`

Recall predefined scenes by name. Scenes are not limited to lights -- they can
control any combination of capabilities.

| Property       | ID       | Datatype | Unit | Format                      | Settable | Retained | Optional | Description            |
| -------------- | -------- | -------- | ---- | --------------------------- | -------- | -------- | -------- | ---------------------- |
| Recall a scene | `recall` | Enum     | --   | Configured scene names      | yes      | no       | no       | Recall a named scene   |

---

#### Shutter

**ID:** `shutter` | **Type:** `hc-smarthome/v2/cap/shutter`

Window shutter/blind position control with directional actions.

| Property         | ID         | Datatype | Unit | Format                        | Settable | Retained | Optional | Description                            |
| ---------------- | ---------- | -------- | ---- | ----------------------------- | -------- | -------- | -------- | -------------------------------------- |
| Shutter position | `position` | Integer  | `%`  | `0:100`                       | yes      | yes      | no       | Current position (0=closed, 100=open)  |
| Control shutter  | `action`   | Enum     | --   | `up`, `down` [, `stop`]       | yes      | no       | no       | Control shutter movement               |

The `stop` variant is included when the device supports it.

---

#### Thermostat

**ID:** `thermostat` | **Type:** `hc-smarthome/v2/cap/thermostat`

Heating/cooling setpoint and mode control. Measured ambient temperature belongs
on a separate `climate` capability node.

| Property                    | ID                | Datatype | Unit | Format                                  | Settable | Retained | Optional | Description                 |
| --------------------------- | ----------------- | -------- | ---- | --------------------------------------- | -------- | -------- | -------- | --------------------------- |
| Set target temperature      | `set-temperature` | Float    | `°C` | Float range (default: `5.0:32.0`/`0.5`) | yes      | yes      | no       | Target temperature setpoint |
| Valve opening level         | `valve`           | Integer  | `%`  | `0:100`                                 | no       | yes      | yes      | Current valve opening       |
| Window open detected        | `window-open`     | Boolean  | --   | `false="closed"`, `true="open"`         | no       | yes      | yes      | Window-open condition       |
| Boost mode active           | `boost-state`     | Boolean  | --   | --                                      | yes      | yes      | yes      | Boost heating active        |
| Seconds remaining for boost | `boost-time`      | Integer  | `s`  | `0:`                                    | no       | no       | yes      | Remaining boost seconds     |
| Mode                        | `mode`            | Enum     | --   | Configured mode values                  | yes      | yes      | yes      | Operating mode              |

All possible mode values: `off`, `auto`, `manual`, `party`, `boost`, `cool`,
`heat`, `emergency-heating`, `precooling`, `fan-only`, `dry`, `sleep`.

---

#### Lock

**ID:** `lock` | **Type:** `hc-smarthome/v2/cap/lock`

Lock/unlock control for door locks, safes, or other locking mechanisms.

| Property    | ID       | Datatype | Unit | Format                              | Settable | Retained | Optional | Description         |
| ----------- | -------- | -------- | ---- | ----------------------------------- | -------- | -------- | -------- | ------------------- |
| Lock state  | `state`  | Boolean  | --   | `false="unlocked"`, `true="locked"` | yes      | yes      | no       | Current lock state  |
| Lock action | `action` | Enum     | --   | `lock`, `unlock`, `toggle`          | yes      | no       | no       | Lock control action |

---

#### Valve

**ID:** `valve` | **Type:** `hc-smarthome/v2/cap/valve`

Binary valve open/close control for water or gas shutoff valves.

| Property    | ID      | Datatype | Unit | Format                          | Settable | Retained | Optional | Description         |
| ----------- | ------- | -------- | ---- | ------------------------------- | -------- | -------- | -------- | ------------------- |
| Valve state | `state` | Boolean  | --   | `false="closed"`, `true="open"` | yes      | yes      | no       | Current valve state |

---

### Sensor Capabilities

#### Climate

**ID:** `climate` | **Type:** `hc-smarthome/v2/cap/climate`

Climate conditions sensing: temperature, humidity, and atmospheric pressure.
All properties are optional. Read-only.

| Property            | ID            | Datatype | Unit  | Format | Settable | Retained | Optional | Description          |
| ------------------- | ------------- | -------- | ----- | ------ | -------- | -------- | -------- | -------------------- |
| Current temperature | `temperature` | Float    | `°C`  | --     | no       | yes      | yes      | Ambient temperature  |
| Current humidity    | `humidity`    | Integer  | `%`   | --     | no       | yes      | yes      | Relative humidity    |
| Current pressure    | `pressure`    | Float    | `kPa` | --     | no       | yes      | yes      | Atmospheric pressure |

---

#### Motion

**ID:** `motion` | **Type:** `hc-smarthome/v2/cap/motion`

Motion detection with optional ambient light level. Read-only.

| Property            | ID       | Datatype | Unit | Format                                 | Settable | Retained | Optional | Description           |
| ------------------- | -------- | -------- | ---- | -------------------------------------- | -------- | -------- | -------- | --------------------- |
| Motion detected     | `motion` | Boolean  | --   | `false="no-motion"`, `true="motion"`   | no       | yes      | no       | Motion detected       |
| Current light level | `lux`    | Integer  | `lx` | --                                     | no       | yes      | yes      | Ambient light level   |

---

#### Vibration

**ID:** `vibration` | **Type:** `hc-smarthome/v2/cap/vibration`

Vibration detection with optional strength measurement. Read-only.

| Property           | ID                   | Datatype | Unit | Format                                         | Settable | Retained | Optional | Description         |
| ------------------ | -------------------- | -------- | ---- | ---------------------------------------------- | -------- | -------- | -------- | ------------------- |
| Vibration detected | `vibration`          | Boolean  | --   | `false="no-vibration"`, `true="vibration"`     | no       | yes      | no       | Vibration detected  |
| Vibration strength | `vibration-strength` | Integer  | --   | --                                             | no       | yes      | yes      | Vibration intensity |

---

#### Contact

**ID:** `contact` | **Type:** `hc-smarthome/v2/cap/contact`

Binary open/close contact sensor (door sensors, window sensors). Read-only.

| Property        | ID      | Datatype | Unit | Format                          | Settable | Retained | Optional | Description           |
| --------------- | ------- | -------- | ---- | ------------------------------- | -------- | -------- | -------- | --------------------- |
| Open/Close state | `state` | Boolean  | --   | `false="closed"`, `true="open"` | no       | yes      | no       | Contact open or closed |

---

#### Water

**ID:** `water` | **Type:** `hc-smarthome/v2/cap/water`

Water leak/flood detection. Read-only.

| Property        | ID         | Datatype | Unit | Format                                       | Settable | Retained | Optional | Description    |
| --------------- | ---------- | -------- | ---- | -------------------------------------------- | -------- | -------- | -------- | -------------- |
| Water detection | `detected` | Boolean  | --   | `false="no water"`, `true="water detected"`  | no       | yes      | no       | Water detected |

---

#### Tilt

**ID:** `tilt` | **Type:** `hc-smarthome/v2/cap/tilt`

Binary tilt detection. Read-only.

| Property     | ID      | Datatype | Unit | Format                                   | Settable | Retained | Optional | Description      |
| ------------ | ------- | -------- | ---- | ---------------------------------------- | -------- | -------- | -------- | ---------------- |
| Tilted state | `state` | Boolean  | --   | `false="not tilted"`, `true="tilted"`    | no       | yes      | no       | Device is tilted |

---

#### Orientation

**ID:** `orientation` | **Type:** `hc-smarthome/v2/cap/orientation`

3-axis orientation sensor reporting rotation angles and tilt. Read-only.

| Property        | ID              | Datatype | Unit | Format | Settable | Retained | Optional | Description           |
| --------------- | --------------- | -------- | ---- | ------ | -------- | -------- | -------- | --------------------- |
| Rotation X-Axis | `orientation-x` | Integer  | `°`  | --     | no       | yes      | no       | X axis rotation angle |
| Rotation Y-Axis | `orientation-y` | Integer  | `°`  | --     | no       | yes      | no       | Y axis rotation angle |
| Rotation Z-Axis | `orientation-z` | Integer  | `°`  | --     | no       | yes      | no       | Z axis rotation angle |
| Tilt angle      | `tilt`          | Integer  | `°`  | --     | no       | yes      | no       | Tilt angle            |

---

### Infrastructure Capabilities

#### Button

**ID:** `button` | **Type:** `hc-smarthome/v2/cap/button`

Physical push-button reporting press events. Output-only (not retained, not
settable).

| Property            | ID       | Datatype | Unit | Format                 | Settable | Retained | Optional | Description        |
| ------------------- | -------- | -------- | ---- | ---------------------- | -------- | -------- | -------- | ------------------ |
| Button action event | `action` | Enum     | --   | Configured actions     | no       | no       | no       | Button press event |

All possible action values: `press`, `long-press`, `double-press`, `release`,
`long-release`, `continuous`.

---

#### Powermeter

**ID:** `powermeter` | **Type:** `hc-smarthome/v2/cap/powermeter`

Electrical power metering. Read-only.

| Property    | ID            | Datatype | Unit | Format | Settable | Retained | Optional | Description              |
| ----------- | ------------- | -------- | ---- | ------ | -------- | -------- | -------- | ------------------------ |
| Power       | `power`       | Float    | `W`  | `0.0:` | no       | yes      | no       | Current power draw       |
| Current     | `current`     | Float    | `mA` | `0.0:` | no       | yes      | yes      | Electrical current       |
| Voltage     | `voltage`     | Float    | `V`  | `0.0:` | no       | yes      | yes      | Voltage                  |
| Frequency   | `frequency`   | Float    | `Hz` | `0.0:` | no       | yes      | yes      | AC frequency             |
| Consumption | `consumption` | Float    | `Wh` | `0.0:` | no       | yes      | yes      | Total energy consumption |

---

### Media Capabilities

#### Mediaplayer

**ID:** `mediaplayer` | **Type:** `hc-smarthome/v2/cap/mediaplayer`

Playback transport control and mode settings. The `action` property triggers
transport commands; `state` reports the current playback state. `shuffle`
and `repeat` use a tri-state enum (`on`, `off`, `disabled`) where `disabled`
indicates the feature is temporarily unavailable for the current media
context (e.g. no shuffle during radio playback).

| Property     | ID        | Datatype | Unit | Format                                                   | Settable | Retained | Optional | Description         |
| ------------ | --------- | -------- | ---- | -------------------------------------------------------- | -------- | -------- | -------- | ------------------- |
| Player action| `action`  | Enum     | --   | `play,pause`[,`stop`][,`next`][,`previous`][,`forward`][,`rewind`] | yes | no  | no       | Transport command   |
| Play state   | `state`   | Enum     | --   | `playing,paused,stopped`                                 | no       | yes      | no       | Current play state  |
| Shuffle mode | `shuffle` | Enum     | --   | `on,off,disabled`                                        | yes      | yes      | yes      | Shuffle mode        |
| Repeat mode  | `repeat`  | Enum     | --   | `on,off,disabled`                                        | yes      | yes      | yes      | Repeat mode         |

`play` and `pause` are always present. Additional actions (`stop`, `next`,
`previous`, `forward`, `rewind`) are included based on device capabilities.

---

#### Media Info

**ID:** `media-info` | **Type:** `hc-smarthome/v2/cap/media-info`

Now-playing metadata and progress. The `title` property is always present;
all other properties are optional. `subtitle` and `description` are generic
text lines -- the bridge maps media-specific information (artist, album,
director, episode, etc.) into them. The optional `metadata` property carries
extended structured metadata as a JSON object with well-known keys.

| Property    | ID            | Datatype | Unit | Format | Settable | Retained | Optional | Description                         |
| ----------- | ------------- | -------- | ---- | ------ | -------- | -------- | -------- | ----------------------------------- |
| Title       | `title`       | String   | --   | --     | no       | yes      | no       | Track/media title                   |
| Subtitle    | `subtitle`    | String   | --   | --     | no       | yes      | yes      | Secondary text line (artist, etc.)  |
| Description | `description` | String   | --   | --     | no       | yes      | yes      | Additional context line             |
| Artwork     | `artwork`     | String   | --   | --     | no       | yes      | yes      | Artwork/cover URL                   |
| Progress    | `progress`    | Integer  | `s`  | --     | yes      | yes      | yes      | Current position (seconds)          |
| Length      | `length`      | Integer  | `s`  | --     | no       | yes      | yes      | Total duration (seconds)            |
| Seekable    | `seekable`    | Boolean  | --   | --     | no       | yes      | yes      | Whether seeking is supported        |
| Metadata    | `metadata`    | JSON     | --   | --     | no       | yes      | yes      | Extended metadata (well-known keys) |

When `seekable` is `false`, the player SHOULD ignore `/set` commands on
`progress`. UIs SHOULD grey out the seek control when `seekable` is absent
or `false`.

---

#### Volume

**ID:** `volume` | **Type:** `hc-smarthome/v2/cap/volume`

Audio volume control with optional mute. The `mute` property uses a
tri-state enum (`on`, `off`, `disabled`) following the same convention as
the mediaplayer capability's shuffle/repeat properties.

| Property     | ID    | Datatype | Unit | Format            | Settable | Retained | Optional | Description  |
| ------------ | ----- | -------- | ---- | ----------------- | -------- | -------- | -------- | ------------ |
| Volume level | `level` | Integer  | `%`  | `0:100`           | yes      | yes      | no       | Volume level |
| Mute         | `mute`  | Enum     | --   | `on,off,disabled` | yes      | yes      | yes      | Mute state   |

---

#### Battery

**ID:** `battery` | **Type:** `hc-smarthome/v2/cap/battery`

Battery health readings. All properties are optional. Read-only.

| Property        | ID        | Datatype | Unit | Format  | Settable | Retained | Optional | Description               |
| --------------- | --------- | -------- | ---- | ------- | -------- | -------- | -------- | ------------------------- |
| Battery level   | `level`   | Integer  | `%`  | `0:100` | no       | yes      | yes      | Battery charge percentage |
| Battery voltage | `voltage` | Integer  | `mV` | --      | no       | yes      | yes      | Raw battery voltage       |

Binary battery conditions (low, critical) are signaled via
[Alerts](#alerts) (`hc-battery-low`, `hc-battery-critical`).

---

#### Link

**ID:** `link` | **Type:** `hc-smarthome/v2/cap/link`

Communication link quality readings. All properties are optional. Read-only.

| Property        | ID          | Datatype | Unit  | Format  | Settable | Retained | Optional | Description                        |
| --------------- | ----------- | -------- | ----- | ------- | -------- | -------- | -------- | ---------------------------------- |
| Signal strength | `signal`    | Integer  | `dBm` | --      | no       | yes      | yes      | RF signal strength (RSSI)          |
| Link quality    | `quality`   | Integer  | --    | `0:255` | no       | yes      | yes      | Link quality indicator (LQI)       |
| Last seen       | `last-seen` | Datetime | --    | --      | no       | yes      | yes      | Timestamp of last received message |

Reachability conditions are signaled via [Alerts](#alerts)
(`hc-unreachable`, `hc-update-overdue`).

---

## Alerts

The alert system provides a lightweight mechanism for devices to signal
health conditions without dedicated capability nodes. Alert IDs follow Homie
topic ID rules and use the `hc-` prefix for well-known alerts. Devices MAY
publish custom alert IDs without the prefix.

| Alert ID              | Description                                    |
| --------------------- | ---------------------------------------------- |
| `hc-battery-low`      | Battery level is low                           |
| `hc-battery-critical` | Battery level is critically low                |
| `hc-unreachable`      | Device is unreachable                          |
| `hc-update-overdue`   | No update received for an extended period      |
| `hc-config-error`     | Configuration error on the device              |
| `hc-sensor-fault`     | Sensor reporting faulty or out-of-range values |
| `hc-tamper`           | Physical tamper detected                       |
| `hc-comm-error`       | Communication error with underlying protocol   |

---
---

# Rust Library Reference

The `hc-homie5-smarthome` crate provides typed Rust implementations of the
above specification.

## Architecture

Each capability is implemented as a Rust module (`*_node.rs`) providing:

- **Config struct** -- serde-compatible configuration with `#[serde(default)]`
  controlling which optional properties are included and whether properties
  are settable.
- **Builder** -- generates a `HomieNodeDescription` from config, with the
  correct type string, property metadata, and constraints.
- **Publisher** -- emits state updates as `homie5::client::Publish` messages.
- **SetCommandParser** -- parses incoming `/set` commands into typed event
  enums via the `ParseOutcome<T>` result type (`NoMatch` / `Parsed(T)` /
  `Invalid(ParseError)`).

## Configuration

All config structs implement `Default` and `Deserialize` with
`#[serde(default)]`, so missing fields use sensible defaults.

| Capability  | Config Struct           | Key Fields                                               |
| ----------- | ----------------------- | -------------------------------------------------------- |
| Switch      | `SwitchNodeConfig`      | `settable`                                               |
| Level       | `LevelNodeConfig`       | `settable`, `step_action`                                |
| Color       | `ColorNodeConfig`       | `settable`, `color_formats`, `ctmin`, `ctmax`            |
| Scene       | `SceneNodeConfig`       | `scenes`, `settable`                                     |
| Shutter     | `ShutterNodeConfig`     | `can_stop`                                               |
| Thermostat  | `ThermostatNodeConfig`  | `unit`, `valve`, `windowopen`, `boost_state`, `boost_time`, `mode`, `modes`, `temp_range` |
| Lock        | `LockNodeConfig`        | `settable`                                               |
| Valve       | `ValveNodeConfig`       | `settable`                                               |
| Climate     | `ClimateNodeConfig`     | `temperature`, `humidity`, `pressure`, `temp_unit`       |
| Motion      | `MotionNodeConfig`      | `lux`                                                    |
| Vibration   | `VibrationNodeConfig`   | `vibration_strength`                                     |
| Button      | `ButtonNodeConfig`      | `actions`                                                |
| Powermeter  | `PowermeterNodeConfig`  | `current`, `voltage`, `frequency`, `consumption`         |
| Mediaplayer | `MediaplayerNodeConfig` | `next`, `previous`, `forward`, `rewind`, `stop`, `shuffle`, `repeat` |
| Media Info  | `MediaInfoNodeConfig`   | `subtitle`, `description`, `artwork`, `progress`, `length`, `seekable`, `metadata` |
| Volume      | `VolumeNodeConfig`      | `mute`                                                   |
| Battery     | `BatteryNodeConfig`     | `level`, `voltage`                                       |
| Link        | `LinkNodeConfig`        | `signal`, `quality`, `last_seen`                         |

## Code Examples

### Creating a capability node

```rust
use hc_homie5_smarthome::switch_node::*;
use hc_homie5_smarthome::SetCommandParser;

// 1. Configure
let config = SwitchNodeConfig::default();

// 2. Build description + publisher
let (node_desc, publisher) = SwitchNodeBuilder::new(&config)
    .build_with_publisher("switch-1".try_into().unwrap(), &client);

// 3. Publish state
let publish = publisher.state(true);

// 4. Parse incoming /set commands
match publisher.parse_set_event(&device_description, &incoming_event) {
    ParseOutcome::Parsed(SwitchNodeSetEvents::State(on)) => { /* handle */ }
    ParseOutcome::Parsed(SwitchNodeSetEvents::Action(action)) => { /* handle */ }
    ParseOutcome::NoMatch => { /* not for this node */ }
    ParseOutcome::Invalid(err) => { /* log error */ }
}
```

### Composing a device from capabilities

```rust
use hc_homie5_smarthome::{
    switch_node::*, level_node::*, color_node::*, battery_node::*, link_node::*,
};
use homie5::device_description::DeviceDescriptionBuilder;

// A full-color light = switch + level + color + battery + link
let desc = DeviceDescriptionBuilder::new()
    .name("Living Room Lamp")
    .add_node(
        SWITCH_NODE_DEFAULT_ID,
        SwitchNodeBuilder::new(&Default::default()).build(),
    )
    .add_node(
        LEVEL_NODE_DEFAULT_ID,
        LevelNodeBuilder::new(&Default::default()).build(),
    )
    .add_node(
        COLOR_NODE_DEFAULT_ID,
        ColorNodeBuilder::new(&Default::default()).build(),
    )
    .add_node(
        BATTERY_NODE_DEFAULT_ID,
        BatteryNodeBuilder::new(&Default::default()).build(),
    )
    .add_node(
        LINK_NODE_DEFAULT_ID,
        LinkNodeBuilder::new(&Default::default()).build(),
    )
    .build();
```



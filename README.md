# Homecontrol Homie5 Smarthome Nodes

A Rust library of typed smart home node definitions for the [Homie 5 MQTT convention](https://homieiot.github.io/).
Each node provides a builder for generating a Homie device description, a publisher for emitting state updates, and (where applicable) a `SetCommandParser` for parsing incoming `/set` commands into typed events.

All nodes are namespaced under `homie-homecontrol/v1/type=<name>`.

## How to read the tables

| Column       | Meaning |
|--------------|---------|
| **Property** | Human-readable name shown in the Homie device description. |
| **ID**       | Homie property ID used in MQTT topics. |
| **Datatype** | Homie datatype (`Boolean`, `Integer`, `Float`, `Enum`, `Color`, `Datetime`). |
| **Unit**     | Measurement unit string published in the property description. `--` means none. |
| **Format**   | Homie format constraint (ranges, enum values, boolean labels). `--` means none. |
| **Settable** | Whether the property accepts `/set` commands. `config` means the config struct controls this. |
| **Retained** | Whether the MQTT message is published with the retain flag. |
| **Optional** | `no` = always present. Otherwise shows the config field that gates this property (property only exists when that field is `true`). |

---

## Actuators

### Switch Node

**Default ID:** `switch` | **Type:** `homie-homecontrol/v1/type=switch` | **Config:** `SwitchNodeConfig`

On/Off switch with a toggle action. Used for binary actuators like lights, relays, or power outlets.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| On/Off state | `state` | Boolean | -- | `false="off"`, `true="on"` | config | yes | no | Current on/off state |
| Change state | `action` | Enum | -- | `toggle` | config | no | no | Trigger a toggle action |

Config fields: `settable: bool` (default `true`) -- controls whether both properties accept `/set` commands.

---

### Dimmer Node

**Default ID:** `dimmer` | **Type:** `homie-homecontrol/v1/type=dimmer` | **Config:** `DimmerNodeConfig`

Brightness level control with step-up/step-down actions. Used for dimmable lights or fan speed controls.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Brightness Level | `brightness` | Integer | `%` | `0:100` | config | yes | no | Current brightness percentage |
| Change Brightness | `action` | Enum | -- | `brighter`, `darker` | config | no | no | Step brightness up or down |

Config fields: `settable: bool` (default `true`).

---

### Shutter Node

**Default ID:** `shutter` | **Type:** `homie-homecontrol/v1/type=shutter` | **Config:** `ShutterNodeConfig`

Window shutter/blind position control with directional actions. The `stop` action is included when `can_stop` is enabled.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Shutter position | `position` | Integer | `%` | `0:100` | yes | yes | no | Current position percentage (0 = closed, 100 = open) |
| Control Shutter | `action` | Enum | -- | `up`, `down` [, `stop`] | yes | no | no | Control shutter movement |

Config fields: `can_stop: bool` (default `true`) -- when `true`, the `stop` variant is added to the action enum. Both properties are always settable.

---

### Colorlight Node

**Default ID:** `colorlight` | **Type:** `homie-homecontrol/v1/type=colorlight` | **Config:** `ColorlightNodeConfig`

Color light control supporting color values (RGB/HSV/XYZ) and color temperature in mireds.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Color | `color` | Color | -- | Color formats from config (default: `rgb`) | config | yes | no | Current light color |
| Color temperature | `color-temperature` | Integer | -- | `{ctmin}:{ctmax}` (default: `153:555`) | config | yes | no | Color temperature in mireds |

Config fields: `settable: bool` (default `true`), `color_formats: Vec<ColorFormat>` (default `[Rgb]`), `ctmin: i64` (default `153`), `ctmax: i64` (default `555`).

---

### Light Scene Node

**Default ID:** `scenes` | **Type:** `homie-homecontrol/v1/type=lightscene` | **Config:** `LightSceneNodeConfig`

Recall predefined light scenes by name. The list of available scenes is defined in the config.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Recall a scene | `recall` | Enum | -- | Scene names from config | config | no | no | Recall a named light scene |

Config fields: `scenes: Vec<String>` (default `[]`), `settable: bool` (default `false`).

---

### Thermostat Node

**Default ID:** `thermostat` | **Type:** `homie-homecontrol/v1/type=thermostat` | **Config:** `ThermostatNodeConfig`

Heating/cooling thermostat with target temperature, valve position, operating mode, boost mode, and window-open detection. Most properties are optional and controlled by config flags.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Set target temperature | `set-temperature` | Float | config (`°C`) | FloatRange from config (default: `5.0:32.0` step `0.5`) | yes | yes | no | Target temperature setpoint |
| Valve opening level | `valve` | Integer | `%` | `0:100` | no | yes | `config.valve` (default `true`) | Current valve opening percentage |
| Window open detected | `window-open` | Boolean | -- | `false="closed"`, `true="open"` | no | yes | `config.windowopen` (default `true`) | Window-open condition detected |
| Boost mode active | `boost-state` | Boolean | -- | -- | yes | yes | `config.boost_state` (default `true`) | Whether boost heating is active |
| Seconds remaining for boost | `boost-time` | Integer | `s` | `0:` (min 0, no max) | no | no | `config.boost_time` (default `true`) | Remaining seconds in boost mode |
| Mode | `mode` | Enum | -- | Modes from config (default: `auto`, `manual`) | yes | yes | `config.mode` (default `true`) | Thermostat operating mode |

All possible mode values: `off`, `auto`, `manual`, `party`, `boost`, `cool`, `heat`, `emergency-heating`, `precooling`, `fan-only`, `dry`, `sleep`.

Config fields: `unit: String` (default `"°C"`), `valve: bool`, `windowopen: bool`, `boost_state: bool`, `boost_time: bool`, `mode: bool`, `modes: Vec<ThermostatNodeModes>`, `temp_range: FloatRange`.

---

## Sensors

### Weather Node

**Default ID:** `weather` | **Type:** `homie-homecontrol/v1/type=weather` | **Config:** `WeatherNodeConfig`

Climate sensor reporting temperature, humidity, and atmospheric pressure. All three properties are individually config-gated. Read-only -- no `/set` commands.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Current temperature | `temperature` | Float | config (`°C`) | -- | no | yes | `config.temperature` (default `true`) | Ambient temperature |
| Current humidity | `humidity` | Integer | `%` | -- | no | yes | `config.humidity` (default `true`) | Relative humidity |
| Current pressure | `pressure` | Float | `kPa` | -- | no | yes | `config.pressure` (default `false`) | Atmospheric pressure |

---

### Motion Node

**Default ID:** `motion` | **Type:** `homie-homecontrol/v1/type=motion` | **Config:** `MotionNodeConfig`

Motion detection sensor with optional ambient light level measurement. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Motion detected | `motion` | Boolean | -- | `false="no-motion"`, `true="motion"` | no | yes | no | Whether motion is currently detected |
| Current light level | `lux` | Integer | `lx` | -- | no | yes | `config.lux` (default `false`) | Ambient light level |

---

### Vibration Node

**Default ID:** `vibration` | **Type:** `homie-homecontrol/v1/type=vibration` | **Config:** `VibrationNodeConfig`

Vibration detection sensor with optional strength measurement. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Vibration detected | `vibration` | Boolean | -- | `false="no-vibration"`, `true="vibration"` | no | yes | no | Whether vibration is detected |
| Vibration strength | `vibration-strength` | Integer | -- | -- | no | yes | `config.vibration_strength` (default `true`) | Intensity of detected vibration |

---

### Contact Node

**Default ID:** `contact` | **Type:** `homie-homecontrol/v1/type=contact` | **No config**

Binary open/close contact sensor (door sensors, window sensors, magnetic contacts). Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Open/Close state | `state` | Boolean | -- | `false="closed"`, `true="open"` | no | yes | no | Whether the contact is open or closed |

---

### Water Sensor Node

**Default ID:** `water` | **Type:** `homie-homecontrol/v1/type=water` | **No config**

Water leak/flood detection sensor. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Water detection | `detected` | Boolean | -- | `false="no water"`, `true="water detected"` | no | yes | no | Whether water is detected |

---

### Tilt Node

**Default ID:** `tilt` | **Type:** `homie-homecontrol/v1/type=tilt` | **No config**

Binary tilt detection sensor. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Tilted state | `state` | Boolean | -- | `false="not tilted"`, `true="tilted"` | no | yes | no | Whether the device is tilted |

---

### Orientation Node

**Default ID:** `orientation` | **Type:** `homie-homecontrol/v1/type=orientation` | **No config**

3-axis orientation sensor reporting rotation angles and tilt. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Rotation X-Axis | `orientation-x` | Integer | `°` | -- | no | yes | no | Rotation angle around X axis |
| Rotation Y-Axis | `orientation-y` | Integer | `°` | -- | no | yes | no | Rotation angle around Y axis |
| Rotation Z-Axis | `orientation-z` | Integer | `°` | -- | no | yes | no | Rotation angle around Z axis |
| Tilt angle | `tilt` | Integer | `°` | -- | no | yes | no | Tilt angle |

---

## Infrastructure

### Maintenance Node

**Default ID:** `maintenance` | **Type:** `homie-homecontrol/v1/type=maintenance` | **Config:** `MaintenanceNodeConfig`

Device maintenance information including battery status, reachability, and last update timestamp. All four properties are config-gated. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Low battery indicator | `low-battery` | Boolean | -- | -- | no | yes | `config.low_battery` (default `true`) | Whether battery is low |
| Battery level | `battery-level` | Integer | `%` | `0:100` | no | yes | `config.battery_level` (default `false`) | Battery level percentage |
| Last update | `last-update` | Datetime | -- | -- | no | yes | `config.last_update` (default `true`) | Timestamp of last device update |
| Reachable | `reachable` | Boolean | -- | -- | no | yes | `config.reachable` (default `true`) | Whether device is reachable |

---

### Button Node

**Default ID:** `button` | **Type:** `homie-homecontrol/v1/type=button` | **Config:** `ButtonNodeConfig`

Physical push-button reporting press events. The set of supported actions is configurable. Output-only (not retained, not settable).

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Button action event | `action` | Enum | -- | Actions from config (default: `press`) | no | no | no | Emitted button press event |

All possible action values: `press`, `long-press`, `double-press`, `release`, `long-release`, `continuous`.

---

### Powermeter Node

**Default ID:** `powermeter` | **Type:** `homie-homecontrol/v1/type=powermeter` | **Config:** `PowermeterNodeConfig`

Electrical power metering with current power draw and optional current, voltage, frequency, and consumption readings. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Power | `power` | Float | `W` | `0.0:` (min 0, no max) | no | yes | no | Current power draw |
| Current | `current` | Float | `mA` | `0.0:` (min 0, no max) | no | yes | `config.current` (default `true`) | Electrical current |
| Voltage | `voltage` | Float | `V` | `0.0:` (min 0, no max) | no | yes | `config.voltage` (default `true`) | Voltage reading |
| Frequency | `frequency` | Float | `Hz` | `0.0:` (min 0, no max) | no | yes | `config.frequency` (default `false`) | AC frequency |
| Consumption | `consumption` | Float | `Wh` | `0.0:` (min 0, no max) | no | yes | `config.consumption` (default `true`) | Total energy consumption |

---

## Alerts

The `SmarthomeAlert` enum defines well-known alert IDs for the homecontrol ecosystem. Alert IDs follow Homie topic ID rules and use the `hc-` prefix. Devices may publish custom alert IDs without the `hc-` prefix.

| Alert ID | Description |
|---|---|
| `hc-battery-low` | Battery level is low |
| `hc-battery-critical` | Battery level is critically low |
| `hc-unreachable` | Device is unreachable on the underlying network |
| `hc-update-overdue` | No update received for an extended period |
| `hc-config-error` | Configuration error on the device |
| `hc-sensor-fault` | Sensor reporting faulty or out-of-range readings |
| `hc-tamper` | Physical tamper detected |
| `hc-comm-error` | Communication error with the underlying protocol |

---

## Usage

Each node type follows a common pattern:

1. Create a config (or use `Default`)
2. Build the node description via the builder
3. Use the publisher to emit state updates
4. Optionally parse incoming `/set` commands with `SetCommandParser`

```rust
use hc_homie5_smarthome::switch_node::*;

// 1. Configure
let config = SwitchNodeConfig::default();

// 2. Build
let (node_desc, publisher) = SwitchNodeBuilder::new(&config)
    .build_with_publisher("switch-1".try_into().unwrap(), &client);

// 3. Publish state
let publish = publisher.state(true);

// 4. Parse set commands
use hc_homie5_smarthome::SetCommandParser;
match publisher.parse_set_event(&device_description, &incoming_event) {
    ParseOutcome::Parsed(SwitchNodeSetEvents::State(on)) => { /* handle */ }
    ParseOutcome::Parsed(SwitchNodeSetEvents::Action(action)) => { /* handle */ }
    ParseOutcome::NoMatch => { /* not for this node */ }
    ParseOutcome::Invalid(err) => { /* log error */ }
}
```

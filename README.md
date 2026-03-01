# Homecontrol Homie5 Smarthome Nodes

A Rust library of typed smart home capability node definitions for the [Homie 5 MQTT convention](https://homieiot.github.io/).
Each node represents a **reusable capability** (e.g., on/off control, brightness level, color) that can be composed into devices.
Nodes provide a builder for generating Homie device descriptions, a publisher for emitting state updates, and (where applicable) a `SetCommandParser` for parsing incoming `/set` commands into typed events.

All capability types are namespaced under `homie-homecontrol/v2/type=<name>`.

## Architecture: Node = Capability

This library follows a **capability composition** model rather than a device-type model:

- A **node** represents a single capability (switch, level, color, climate sensing, etc.)
- A **device** composes multiple capability nodes to describe its full feature set
- A wall plug: `switch` node
- A dimmable bulb: `switch` + `level` nodes
- A full color bulb: `switch` + `level` + `color` nodes
- A wall thermostat: `thermostat` + `climate` + `maintenance` nodes

**Device classes** (see below) classify what the physical device *is*, independent of which capability nodes it exposes.

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
| **Optional** | `no` = always present. Otherwise shows the config field that gates this property. |

---

## Actuator Capabilities

### Switch

**Default ID:** `switch` | **Type:** `homie-homecontrol/v2/type=switch` | **Config:** `SwitchNodeConfig`

On/Off control with a toggle action. Used for binary actuators like lights, relays, or power outlets.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| On/Off state | `state` | Boolean | -- | `false="off"`, `true="on"` | config | yes | no | Current on/off state |
| Change state | `action` | Enum | -- | `toggle` | config | no | no | Trigger a toggle action |

Config: `settable: bool` (default `true`).

---

### Level

**Default ID:** `level` | **Type:** `homie-homecontrol/v2/type=level` | **Config:** `LevelNodeConfig`

Level control (0-100%) with optional step actions. Used for brightness, fan speed, volume, or any percentage-based control.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Level | `value` | Integer | `%` | `0:100` | config | yes | no | Current level percentage |
| Step level | `action` | Enum | -- | `step-up`, `step-down` | config | no | `config.step_action` (default `true`) | Relative step change |

Config: `settable: bool` (default `true`), `step_action: bool` (default `true`).

---

### Color

**Default ID:** `color` | **Type:** `homie-homecontrol/v2/type=color` | **Config:** `ColorNodeConfig`

Color control supporting color values (RGB/HSV/XYZ) and color temperature in mireds.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Color | `color` | Color | -- | Color formats from config (default: `rgb`) | config | yes | no | Current color value |
| Color temperature | `color-temperature` | Integer | -- | `{ctmin}:{ctmax}` (default: `153:555`) | config | yes | no | Color temperature in mireds |

Config: `settable: bool` (default `true`), `color_formats: Vec<ColorFormat>` (default `[Rgb]`), `ctmin: i64` (default `153`), `ctmax: i64` (default `555`).

---

### Scene

**Default ID:** `scene` | **Type:** `homie-homecontrol/v2/type=scene` | **Config:** `SceneNodeConfig`

Recall predefined scenes by name. Scenes are not limited to lights -- they can control any combination of capabilities.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Recall a scene | `recall` | Enum | -- | Scene names from config | config | no | no | Recall a named scene |

Config: `scenes: Vec<String>` (default `[]`), `settable: bool` (default `false`).

---

### Shutter

**Default ID:** `shutter` | **Type:** `homie-homecontrol/v2/type=shutter` | **Config:** `ShutterNodeConfig`

Window shutter/blind position control with directional actions. The `stop` action is included when `can_stop` is enabled.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Shutter position | `position` | Integer | `%` | `0:100` | yes | yes | no | Current position (0=closed, 100=open) |
| Control Shutter | `action` | Enum | -- | `up`, `down` [, `stop`] | yes | no | no | Control shutter movement |

Config: `can_stop: bool` (default `true`).

---

### Thermostat

**Default ID:** `thermostat` | **Type:** `homie-homecontrol/v2/type=thermostat` | **Config:** `ThermostatNodeConfig`

Heating/cooling setpoint and mode control. Note: measured ambient temperature belongs on a separate `climate` capability node, not here.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Set target temperature | `set-temperature` | Float | config (`°C`) | FloatRange from config (default: `5.0:32.0` step `0.5`) | yes | yes | no | Target temperature setpoint |
| Valve opening level | `valve` | Integer | `%` | `0:100` | no | yes | `config.valve` (default `true`) | Current valve opening |
| Window open detected | `window-open` | Boolean | -- | `false="closed"`, `true="open"` | no | yes | `config.windowopen` (default `true`) | Window-open condition |
| Boost mode active | `boost-state` | Boolean | -- | -- | yes | yes | `config.boost_state` (default `true`) | Boost heating active |
| Seconds remaining for boost | `boost-time` | Integer | `s` | `0:` (min 0, no max) | no | no | `config.boost_time` (default `true`) | Remaining boost seconds |
| Mode | `mode` | Enum | -- | Modes from config (default: `auto`, `manual`) | yes | yes | `config.mode` (default `true`) | Operating mode |

All possible mode values: `off`, `auto`, `manual`, `party`, `boost`, `cool`, `heat`, `emergency-heating`, `precooling`, `fan-only`, `dry`, `sleep`.

---

### Lock

**Default ID:** `lock` | **Type:** `homie-homecontrol/v2/type=lock` | **Config:** `LockNodeConfig`

Lock/unlock control for door locks, safes, or other locking mechanisms.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Lock state | `state` | Boolean | -- | `false="unlocked"`, `true="locked"` | config | yes | no | Current lock state |
| Lock action | `action` | Enum | -- | `lock`, `unlock`, `toggle` | config | no | no | Lock control action |

Config: `settable: bool` (default `true`).

---

### Valve

**Default ID:** `valve` | **Type:** `homie-homecontrol/v2/type=valve` | **Config:** `ValveNodeConfig`

Binary valve open/close control for water or gas shutoff valves.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Valve state | `state` | Boolean | -- | `false="closed"`, `true="open"` | config | yes | no | Current valve state |

Config: `settable: bool` (default `true`).

---

## Sensor Capabilities

### Climate

**Default ID:** `climate` | **Type:** `homie-homecontrol/v2/type=climate` | **Config:** `ClimateNodeConfig`

Climate conditions sensing: temperature, humidity, and atmospheric pressure. All properties are individually config-gated. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Current temperature | `temperature` | Float | config (`°C`) | -- | no | yes | `config.temperature` (default `true`) | Ambient temperature |
| Current humidity | `humidity` | Integer | `%` | -- | no | yes | `config.humidity` (default `true`) | Relative humidity |
| Current pressure | `pressure` | Float | `kPa` | -- | no | yes | `config.pressure` (default `false`) | Atmospheric pressure |

---

### Motion

**Default ID:** `motion` | **Type:** `homie-homecontrol/v2/type=motion` | **Config:** `MotionNodeConfig`

Motion detection with optional ambient light level. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Motion detected | `motion` | Boolean | -- | `false="no-motion"`, `true="motion"` | no | yes | no | Motion currently detected |
| Current light level | `lux` | Integer | `lx` | -- | no | yes | `config.lux` (default `false`) | Ambient light level |

---

### Vibration

**Default ID:** `vibration` | **Type:** `homie-homecontrol/v2/type=vibration` | **Config:** `VibrationNodeConfig`

Vibration detection with optional strength measurement. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Vibration detected | `vibration` | Boolean | -- | `false="no-vibration"`, `true="vibration"` | no | yes | no | Vibration detected |
| Vibration strength | `vibration-strength` | Integer | -- | -- | no | yes | `config.vibration_strength` (default `true`) | Vibration intensity |

---

### Contact

**Default ID:** `contact` | **Type:** `homie-homecontrol/v2/type=contact` | **No config**

Binary open/close contact sensor (door sensors, window sensors). Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Open/Close state | `state` | Boolean | -- | `false="closed"`, `true="open"` | no | yes | no | Contact open or closed |

---

### Water Sensor

**Default ID:** `water` | **Type:** `homie-homecontrol/v2/type=water` | **No config**

Water leak/flood detection. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Water detection | `detected` | Boolean | -- | `false="no water"`, `true="water detected"` | no | yes | no | Water detected |

---

### Tilt

**Default ID:** `tilt` | **Type:** `homie-homecontrol/v2/type=tilt` | **No config**

Binary tilt detection. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Tilted state | `state` | Boolean | -- | `false="not tilted"`, `true="tilted"` | no | yes | no | Device is tilted |

---

### Orientation

**Default ID:** `orientation` | **Type:** `homie-homecontrol/v2/type=orientation` | **No config**

3-axis orientation sensor reporting rotation angles and tilt. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Rotation X-Axis | `orientation-x` | Integer | `°` | -- | no | yes | no | X axis rotation angle |
| Rotation Y-Axis | `orientation-y` | Integer | `°` | -- | no | yes | no | Y axis rotation angle |
| Rotation Z-Axis | `orientation-z` | Integer | `°` | -- | no | yes | no | Z axis rotation angle |
| Tilt angle | `tilt` | Integer | `°` | -- | no | yes | no | Tilt angle |

---

## Infrastructure Capabilities

### Maintenance

**Default ID:** `maintenance` | **Type:** `homie-homecontrol/v2/type=maintenance` | **Config:** `MaintenanceNodeConfig`

Device health and battery status. All properties are config-gated. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Low battery indicator | `low-battery` | Boolean | -- | -- | no | yes | `config.low_battery` (default `true`) | Battery is low |
| Battery level | `battery-level` | Integer | `%` | `0:100` | no | yes | `config.battery_level` (default `false`) | Battery percentage |
| Last update | `last-update` | Datetime | -- | -- | no | yes | `config.last_update` (default `true`) | Last device update |
| Reachable | `reachable` | Boolean | -- | -- | no | yes | `config.reachable` (default `true`) | Device reachable |

---

### Button

**Default ID:** `button` | **Type:** `homie-homecontrol/v2/type=button` | **Config:** `ButtonNodeConfig`

Physical push-button reporting press events. Output-only (not retained, not settable).

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Button action event | `action` | Enum | -- | Actions from config (default: `press`) | no | no | no | Button press event |

All possible action values: `press`, `long-press`, `double-press`, `release`, `long-release`, `continuous`.

---

### Powermeter

**Default ID:** `powermeter` | **Type:** `homie-homecontrol/v2/type=powermeter` | **Config:** `PowermeterNodeConfig`

Electrical power metering. Read-only.

| Property | ID | Datatype | Unit | Format | Settable | Retained | Optional | Description |
|---|---|---|---|---|---|---|---|---|
| Power | `power` | Float | `W` | `0.0:` (min 0, no max) | no | yes | no | Current power draw |
| Current | `current` | Float | `mA` | `0.0:` (min 0, no max) | no | yes | `config.current` (default `true`) | Electrical current |
| Voltage | `voltage` | Float | `V` | `0.0:` (min 0, no max) | no | yes | `config.voltage` (default `true`) | Voltage |
| Frequency | `frequency` | Float | `Hz` | `0.0:` (min 0, no max) | no | yes | `config.frequency` (default `false`) | AC frequency |
| Consumption | `consumption` | Float | `Wh` | `0.0:` (min 0, no max) | no | yes | `config.consumption` (default `true`) | Total energy consumption |

---

## Alerts

The `SmarthomeAlert` enum defines well-known alert IDs for the homecontrol ecosystem. Alert IDs use the `hc-` prefix. Devices may publish custom alert IDs without the prefix.

| Alert ID | Description |
|---|---|
| `hc-battery-low` | Battery level is low |
| `hc-battery-critical` | Battery level is critically low |
| `hc-unreachable` | Device is unreachable |
| `hc-update-overdue` | No update received for an extended period |
| `hc-config-error` | Configuration error on the device |
| `hc-sensor-fault` | Sensor reporting faulty readings |
| `hc-tamper` | Physical tamper detected |
| `hc-comm-error` | Communication error with underlying protocol |

---

## Device Classes

Device classes classify what the physical device *is*, independent of which capability nodes it exposes.
They are intended for the device-level `type` field in Homie 5 device descriptions and help UIs render appropriate icons and group devices meaningfully.

| Constant | Value | Typical Capabilities |
|---|---|---|
| `DEVICE_CLASS_LIGHT` | `light` | `switch` + `level` + `color` + `scene` |
| `DEVICE_CLASS_OUTLET` | `outlet` | `switch` + `powermeter` |
| `DEVICE_CLASS_THERMOSTAT` | `thermostat` | `thermostat` + `climate` + `maintenance` |
| `DEVICE_CLASS_RADIATOR_VALVE` | `radiator-valve` | `thermostat` + `climate` + `maintenance` |
| `DEVICE_CLASS_CLIMATE_SENSOR` | `climate-sensor` | `climate` + `maintenance` |
| `DEVICE_CLASS_MOTION_SENSOR` | `motion-sensor` | `motion` + `climate` + `maintenance` |
| `DEVICE_CLASS_CONTACT_SENSOR` | `contact-sensor` | `contact` + `maintenance` |
| `DEVICE_CLASS_WATER_SENSOR` | `water-sensor` | `water` + `maintenance` |
| `DEVICE_CLASS_LOCK` | `lock` | `lock` + `maintenance` |
| `DEVICE_CLASS_SHUTTER` | `shutter` | `shutter` + `maintenance` |
| `DEVICE_CLASS_FAN` | `fan` | `switch` + `level` + `maintenance` |
| `DEVICE_CLASS_VALVE` | `valve` | `valve` + `maintenance` |
| `DEVICE_CLASS_BUTTON` | `button` | `button` + `maintenance` |
| `DEVICE_CLASS_SIREN` | `siren` | `switch` + `maintenance` |
| `DEVICE_CLASS_POWERMETER` | `powermeter` | `powermeter` + `maintenance` |

---

## Usage

Each capability node follows a common pattern:

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

### Composing a device

```rust
use hc_homie5_smarthome::{
    switch_node::*, level_node::*, color_node::*, maintenance_node::*,
};
use homie5::device_description::DeviceDescriptionBuilder;

// A full-color light = switch + level + color + maintenance
let desc = DeviceDescriptionBuilder::new()
    .name("Living Room Lamp")
    .add_node(SWITCH_NODE_DEFAULT_ID, SwitchNodeBuilder::new(&Default::default()).build())
    .add_node(LEVEL_NODE_DEFAULT_ID, LevelNodeBuilder::new(&Default::default()).build())
    .add_node(COLOR_NODE_DEFAULT_ID, ColorNodeBuilder::new(&Default::default()).build())
    .add_node(MAINTENANCE_NODE_DEFAULT_ID, MaintenanceNodeBuilder::new(Default::default()).build())
    .build();
```

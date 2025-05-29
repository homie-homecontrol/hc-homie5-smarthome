# Homecontrol Homie5 Smarthome Nodes

# HC Homie5 SmartHome

A Rust implementation of Homie 5.0 convention for smart home devices, providing various node types for home automation.

## Features

- Implements Homie 5.0 MQTT convention
- Supports multiple smart home device types:
    - Buttons
    - Color lights
    - Contacts (door/window sensors)
    - Dimmers
    - Light scenes
    - Maintenance nodes
    - Motion sensors
    - Numeric sensors
    - Orientation sensors
    - Shutters
    - Switches
    - Thermostats
    - Tilt sensors
    - Vibration sensors
    - Water sensors
    - Weather sensors

## Supported Node Types

### Basic Nodes

- **ButtonNode**: Handles press actions (press, long-press, double-press)
- **ContactNode**: Open/close state detection
- **SwitchNode**: On/off control with toggle action
- **WaterSensorNode**: Water detection

### Lighting Nodes

- **ColorlightNode**: RGB color control with temperature settings
- **DimmerNode**: Brightness control (0-100%)
- **LightSceneNode**: Scene recall functionality

### Environmental Nodes

- **MotionNode**: Motion detection with optional lux sensor
- **OrientationNode**: 3-axis orientation sensing
- **TiltNode**: Tilt detection
- **VibrationNode**: Vibration detection with strength measurement
- **WeatherNode**: Temperature, humidity, and pressure sensing

### Control Nodes

- **ShutterNode**: Position control (0-100%) with up/down/stop actions
- **ThermostatNode**: Temperature control with multiple modes (auto, manual, boost)

### Utility Nodes

- **MaintenanceNode**: Device status monitoring (battery, reachability, last update)

## Usage

Each node type follows a similar pattern:

1. Create a builder with configuration
2. Build the node description
3. Use the publisher to send updates

Example for a switch node:

```rust
let config = SwitchNodeConfig::default();
let (node, publisher) = SwitchNodeBuilder::new(&config)
    .build_with_publisher("switch1".try_into().unwrap(), &client);

// Update switch state
publisher.state(true);
```

## Configuration

Each node type has its own configuration struct (e.g., `SwitchNodeConfig`, `DimmerNodeConfig`) that allows customization of:

- Settable properties
- Value ranges
- Available actions
- Unit specifications
- Retention policies

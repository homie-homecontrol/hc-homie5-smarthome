pub mod alerts;
pub mod button_node;
pub mod climate_node;
pub mod color_node;
pub mod contact_node;
pub mod level_node;
pub mod lock_node;
pub mod maintenance_node;
pub mod motion_node;
#[allow(deprecated)]
pub mod numeric_sensor_node;
pub mod orientation_node;
pub mod powermeter_node;
pub mod scene_node;
pub mod shutter_node;
pub mod switch_node;
pub mod thermostat_node;
pub mod tilt_node;
pub mod valve_node;
pub mod vibration_node;
pub mod water_sensor_node;

use std::{fmt, str::FromStr};

use button_node::ButtonNodeConfig;
use climate_node::{ClimateNode, ClimateNodeConfig};
use color_node::{ColorNode, ColorNodeConfig};
use contact_node::ContactNode;
use level_node::{LevelNode, LevelNodeConfig};
use lock_node::LockNodeConfig;
use maintenance_node::{MaintenanceNode, MaintenanceNodeConfig};
use motion_node::{MotionNode, MotionNodeConfig};
#[allow(deprecated)]
use numeric_sensor_node::NumericSensorNode;
use powermeter_node::{PowermeterNode, PowermeterNodeConfig};
use scene_node::SceneNodeConfig;
use serde::{Deserialize, Serialize};
use shutter_node::{ShutterNode, ShutterNodeConfig};
use switch_node::{SwitchNode, SwitchNodeConfig};
use thermostat_node::ThermostatNodeConfig;
use tilt_node::TiltNode;
use valve_node::ValveNodeConfig;
use vibration_node::VibrationNodeConfig;
use water_sensor_node::WaterSensorNode;

/// Helper macro to generate capability type strings (`hc-smarthome/v2/cap/<name>`)
macro_rules! smarthome_cap {
    ($name:expr) => {
        concat!("hc-smarthome/v2/cap/", $name)
    };
}

/// Helper macro to generate device class strings (`hc-smarthome/v2/dc/<name>`)
macro_rules! smarthome_dc {
    ($name:expr) => {
        concat!("hc-smarthome/v2/dc/", $name)
    };
}

/// Helper macro to generate extension capability strings (`hc-smarthome/v2/ext/<name>`)
#[macro_export]
macro_rules! smarthome_ext {
    ($name:expr) => {
        concat!("hc-smarthome/v2/ext/", $name)
    };
}

pub const SMARTHOME_NS: &str = "hc-smarthome/v2";

// ── Capability type constants ───────────────────────────────────────────────

pub const SMARTHOME_CAP_MAINTENANCE: &str = smarthome_cap!("maintenance");
pub const SMARTHOME_CAP_SWITCH: &str = smarthome_cap!("switch");
pub const SMARTHOME_CAP_LEVEL: &str = smarthome_cap!("level");
pub const SMARTHOME_CAP_CONTACT: &str = smarthome_cap!("contact");
pub const SMARTHOME_CAP_CLIMATE: &str = smarthome_cap!("climate");
pub const SMARTHOME_CAP_MOTION: &str = smarthome_cap!("motion");
pub const SMARTHOME_CAP_BUTTON: &str = smarthome_cap!("button");
pub const SMARTHOME_CAP_COLOR: &str = smarthome_cap!("color");
pub const SMARTHOME_CAP_SCENE: &str = smarthome_cap!("scene");
pub const SMARTHOME_CAP_NUMERIC: &str = smarthome_cap!("numeric");
pub const SMARTHOME_CAP_VIBRATION: &str = smarthome_cap!("vibration");
pub const SMARTHOME_CAP_ORIENTATION: &str = smarthome_cap!("orientation");
pub const SMARTHOME_CAP_WATER_SENSOR: &str = smarthome_cap!("water");
pub const SMARTHOME_CAP_SHUTTER: &str = smarthome_cap!("shutter");
pub const SMARTHOME_CAP_TILT: &str = smarthome_cap!("tilt");
pub const SMARTHOME_CAP_THERMOSTAT: &str = smarthome_cap!("thermostat");
pub const SMARTHOME_CAP_POWERMETER: &str = smarthome_cap!("powermeter");
pub const SMARTHOME_CAP_LOCK: &str = smarthome_cap!("lock");
pub const SMARTHOME_CAP_VALVE: &str = smarthome_cap!("valve");

// ── Well-known device class constants ───────────────────────────────────────
//
// These are intended for the device-level `type` field in Homie 5 device
// descriptions.  They classify what the *physical device* is, independent
// of which capability nodes it exposes.

pub const DEVICE_CLASS_LIGHT: &str = smarthome_dc!("light");
pub const DEVICE_CLASS_OUTLET: &str = smarthome_dc!("outlet");
pub const DEVICE_CLASS_THERMOSTAT: &str = smarthome_dc!("thermostat");
pub const DEVICE_CLASS_RADIATOR_VALVE: &str = smarthome_dc!("radiator-valve");
pub const DEVICE_CLASS_CLIMATE_SENSOR: &str = smarthome_dc!("climate-sensor");
pub const DEVICE_CLASS_MOTION_SENSOR: &str = smarthome_dc!("motion-sensor");
pub const DEVICE_CLASS_CONTACT_SENSOR: &str = smarthome_dc!("contact-sensor");
pub const DEVICE_CLASS_WATER_SENSOR: &str = smarthome_dc!("water-sensor");
pub const DEVICE_CLASS_LOCK: &str = smarthome_dc!("lock");
pub const DEVICE_CLASS_SHUTTER: &str = smarthome_dc!("shutter");
pub const DEVICE_CLASS_FAN: &str = smarthome_dc!("fan");
pub const DEVICE_CLASS_VALVE: &str = smarthome_dc!("valve");
pub const DEVICE_CLASS_BUTTON: &str = smarthome_dc!("button");
pub const DEVICE_CLASS_SIREN: &str = smarthome_dc!("siren");
pub const DEVICE_CLASS_POWERMETER: &str = smarthome_dc!("powermeter");

// ── Parse infrastructure ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    UnexpectedMessageType,
    MissingPropertyDescription,
    InvalidHomieValue,
    InvalidVariant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub property_id: String,
    pub payload: String,
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(
        property_id: impl Into<String>,
        payload: impl Into<String>,
        kind: ParseErrorKind,
    ) -> Self {
        Self {
            property_id: property_id.into(),
            payload: payload.into(),
            kind,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "set command parse error ({:?}) for property '{}' and payload '{}'",
            self.kind, self.property_id, self.payload
        )
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseOutcome<T> {
    NoMatch,
    Parsed(T),
    Invalid(ParseError),
}

impl<T> ParseOutcome<T> {
    pub fn ok(self) -> Option<T> {
        match self {
            ParseOutcome::Parsed(value) => Some(value),
            ParseOutcome::NoMatch | ParseOutcome::Invalid(_) => None,
        }
    }

    pub fn into_result(self) -> Result<Option<T>, ParseError> {
        match self {
            ParseOutcome::NoMatch => Ok(None),
            ParseOutcome::Parsed(value) => Ok(Some(value)),
            ParseOutcome::Invalid(err) => Err(err),
        }
    }
}

impl<T> From<ParseOutcome<T>> for Option<T> {
    fn from(value: ParseOutcome<T>) -> Self {
        value.ok()
    }
}

impl<T> From<ParseOutcome<T>> for Result<Option<T>, ParseError> {
    fn from(value: ParseOutcome<T>) -> Self {
        value.into_result()
    }
}

impl<T> From<T> for ParseOutcome<T> {
    fn from(value: T) -> Self {
        ParseOutcome::Parsed(value)
    }
}

pub trait SetCommandParser {
    type Event;

    fn parse_set(
        &self,
        property: &homie5::PropertyRef,
        desc: &homie5::device_description::HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event>;

    fn parse_set_event(
        &self,
        desc: &homie5::device_description::HomieDeviceDescription,
        event: &homie5::Homie5Message,
    ) -> ParseOutcome<Self::Event>;
}

// ── SmarthomeType enum ──────────────────────────────────────────────────────

/// SmarthomeType enum representing the capability node types.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SmarthomeType {
    Switch,
    Level,
    Maintenance,
    Contact,
    Climate,
    Motion,
    Button,
    Color,
    Scene,
    Numeric,
    Vibration,
    Orientation,
    WaterSensor,
    Shutter,
    Tilt,
    Thermostat,
    Powermeter,
    Lock,
    Valve,
}

impl SmarthomeType {
    /// Convert the enum variant into its corresponding string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            SmarthomeType::Switch => SMARTHOME_CAP_SWITCH,
            SmarthomeType::Level => SMARTHOME_CAP_LEVEL,
            SmarthomeType::Maintenance => SMARTHOME_CAP_MAINTENANCE,
            SmarthomeType::Contact => SMARTHOME_CAP_CONTACT,
            SmarthomeType::Climate => SMARTHOME_CAP_CLIMATE,
            SmarthomeType::Motion => SMARTHOME_CAP_MOTION,
            SmarthomeType::Button => SMARTHOME_CAP_BUTTON,
            SmarthomeType::Color => SMARTHOME_CAP_COLOR,
            SmarthomeType::Scene => SMARTHOME_CAP_SCENE,
            SmarthomeType::Numeric => SMARTHOME_CAP_NUMERIC,
            SmarthomeType::Vibration => SMARTHOME_CAP_VIBRATION,
            SmarthomeType::Orientation => SMARTHOME_CAP_ORIENTATION,
            SmarthomeType::WaterSensor => SMARTHOME_CAP_WATER_SENSOR,
            SmarthomeType::Shutter => SMARTHOME_CAP_SHUTTER,
            SmarthomeType::Tilt => SMARTHOME_CAP_TILT,
            SmarthomeType::Thermostat => SMARTHOME_CAP_THERMOSTAT,
            SmarthomeType::Powermeter => SMARTHOME_CAP_POWERMETER,
            SmarthomeType::Lock => SMARTHOME_CAP_LOCK,
            SmarthomeType::Valve => SMARTHOME_CAP_VALVE,
        }
    }

    /// Create a SmarthomeType from a string containing a constant value.
    pub fn from_constant(value: &str) -> Option<Self> {
        match value {
            SMARTHOME_CAP_SWITCH => Some(SmarthomeType::Switch),
            SMARTHOME_CAP_LEVEL => Some(SmarthomeType::Level),
            SMARTHOME_CAP_MAINTENANCE => Some(SmarthomeType::Maintenance),
            SMARTHOME_CAP_CONTACT => Some(SmarthomeType::Contact),
            SMARTHOME_CAP_CLIMATE => Some(SmarthomeType::Climate),
            SMARTHOME_CAP_MOTION => Some(SmarthomeType::Motion),
            SMARTHOME_CAP_BUTTON => Some(SmarthomeType::Button),
            SMARTHOME_CAP_COLOR => Some(SmarthomeType::Color),
            SMARTHOME_CAP_SCENE => Some(SmarthomeType::Scene),
            SMARTHOME_CAP_NUMERIC => Some(SmarthomeType::Numeric),
            SMARTHOME_CAP_VIBRATION => Some(SmarthomeType::Vibration),
            SMARTHOME_CAP_ORIENTATION => Some(SmarthomeType::Orientation),
            SMARTHOME_CAP_WATER_SENSOR => Some(SmarthomeType::WaterSensor),
            SMARTHOME_CAP_SHUTTER => Some(SmarthomeType::Shutter),
            SMARTHOME_CAP_TILT => Some(SmarthomeType::Tilt),
            SMARTHOME_CAP_THERMOSTAT => Some(SmarthomeType::Thermostat),
            SMARTHOME_CAP_POWERMETER => Some(SmarthomeType::Powermeter),
            SMARTHOME_CAP_LOCK => Some(SmarthomeType::Lock),
            SMARTHOME_CAP_VALVE => Some(SmarthomeType::Valve),
            _ => None,
        }
    }
}

impl FromStr for SmarthomeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SmarthomeType::from_constant(s).ok_or(())
    }
}

impl fmt::Display for SmarthomeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for SmarthomeType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SmarthomeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <&str>::deserialize(deserializer)?;
        SmarthomeType::from_constant(value)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid smarthome type: {value}")))
    }
}

// ── Convenience config/node enums ───────────────────────────────────────────

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SmarthomeProperyConfig {
    Button(ButtonNodeConfig),
    Color(ColorNodeConfig),
    Level(LevelNodeConfig),
    Lock(LockNodeConfig),
    Scene(SceneNodeConfig),
    Maintenance(MaintenanceNodeConfig),
    Motion(MotionNodeConfig),
    Shutter(ShutterNodeConfig),
    Switch(SwitchNodeConfig),
    Thermostat(ThermostatNodeConfig),
    Valve(ValveNodeConfig),
    Vibration(VibrationNodeConfig),
    Climate(ClimateNodeConfig),
    Powermeter(PowermeterNodeConfig),
}

#[derive(Debug)]
#[allow(deprecated)]
pub enum SmarthomeNode {
    MaintenanceNode(MaintenanceNode),
    SwitchNode(SwitchNode),
    LevelNode(LevelNode),
    ClimateNode(ClimateNode),
    ContactNode(ContactNode),
    MotionNode(MotionNode),
    ColorNode(ColorNode),
    #[deprecated(note = "Use dedicated typed capabilities (climate, powermeter) instead")]
    NumericSensorNode(NumericSensorNode),
    WaterSensor(WaterSensorNode),
    ShutterNode(ShutterNode),
    TiltNode(TiltNode),
    Powermeter(PowermeterNode),
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod parse_outcome_tests {
    use super::*;

    #[test]
    fn parse_outcome_ok_returns_only_parsed_value() {
        assert_eq!(ParseOutcome::Parsed(42).ok(), Some(42));
        assert_eq!(ParseOutcome::<i32>::NoMatch.ok(), None);
        assert_eq!(
            ParseOutcome::<i32>::Invalid(ParseError::new(
                "x",
                "y",
                ParseErrorKind::InvalidHomieValue
            ))
            .ok(),
            None
        );
    }

    #[test]
    fn parse_outcome_into_result_preserves_invalid_error() {
        let no_match = ParseOutcome::<i32>::NoMatch
            .into_result()
            .expect("no match should be ok");
        assert_eq!(no_match, None);

        let parsed = ParseOutcome::Parsed(7)
            .into_result()
            .expect("parsed should be ok");
        assert_eq!(parsed, Some(7));

        let err = ParseOutcome::<i32>::Invalid(ParseError::new(
            "state",
            "not-bool",
            ParseErrorKind::InvalidHomieValue,
        ))
        .into_result()
        .expect_err("invalid should return error");
        assert_eq!(err.kind, ParseErrorKind::InvalidHomieValue);
        assert_eq!(err.property_id, "state");
    }

    #[test]
    fn parse_outcome_from_impls_work() {
        let parsed: ParseOutcome<i32> = 5.into();
        assert_eq!(parsed, ParseOutcome::Parsed(5));

        let no_match_option: Option<i32> = ParseOutcome::NoMatch.into();
        assert_eq!(no_match_option, None);

        let parsed_option: Option<i32> = ParseOutcome::Parsed(11).into();
        assert_eq!(parsed_option, Some(11));

        let parsed_result: Result<Option<i32>, ParseError> = ParseOutcome::Parsed(12).into();
        assert_eq!(parsed_result.expect("parsed should map to ok"), Some(12));
    }
}

#[cfg(test)]
mod config_serde_default_tests {
    use super::*;

    #[test]
    fn empty_object_deserializes_to_default_for_all_node_configs() {
        let switch: SwitchNodeConfig =
            serde_json::from_str("{}").expect("switch config must deserialize");
        assert_eq!(switch, SwitchNodeConfig::default());

        let level: LevelNodeConfig =
            serde_json::from_str("{}").expect("level config must deserialize");
        assert_eq!(level, LevelNodeConfig::default());

        let shutter: ShutterNodeConfig =
            serde_json::from_str("{}").expect("shutter config must deserialize");
        assert_eq!(shutter, ShutterNodeConfig::default());

        let color: ColorNodeConfig =
            serde_json::from_str("{}").expect("color config must deserialize");
        assert_eq!(color, ColorNodeConfig::default());

        let scene: SceneNodeConfig =
            serde_json::from_str("{}").expect("scene config must deserialize");
        assert_eq!(scene, SceneNodeConfig::default());

        let thermostat: ThermostatNodeConfig =
            serde_json::from_str("{}").expect("thermostat config must deserialize");
        assert_eq!(thermostat, ThermostatNodeConfig::default());

        let climate: ClimateNodeConfig =
            serde_json::from_str("{}").expect("climate config must deserialize");
        assert_eq!(climate, ClimateNodeConfig::default());

        let motion: MotionNodeConfig =
            serde_json::from_str("{}").expect("motion config must deserialize");
        assert_eq!(motion, MotionNodeConfig::default());

        let vibration: VibrationNodeConfig =
            serde_json::from_str("{}").expect("vibration config must deserialize");
        assert_eq!(vibration, VibrationNodeConfig::default());

        let maintenance: MaintenanceNodeConfig =
            serde_json::from_str("{}").expect("maintenance config must deserialize");
        assert_eq!(maintenance, MaintenanceNodeConfig::default());

        let button: ButtonNodeConfig =
            serde_json::from_str("{}").expect("button config must deserialize");
        assert_eq!(button, ButtonNodeConfig::default());

        let powermeter: PowermeterNodeConfig =
            serde_json::from_str("{}").expect("powermeter config must deserialize");
        assert_eq!(powermeter, PowermeterNodeConfig::default());

        let lock: LockNodeConfig =
            serde_json::from_str("{}").expect("lock config must deserialize");
        assert_eq!(lock, LockNodeConfig::default());

        let valve: ValveNodeConfig =
            serde_json::from_str("{}").expect("valve config must deserialize");
        assert_eq!(valve, ValveNodeConfig::default());
    }

    #[test]
    fn partial_config_deserialization_keeps_defaults_for_missing_fields() {
        let thermostat: ThermostatNodeConfig = serde_json::from_str(r#"{"unit":"F"}"#)
            .expect("thermostat partial config must deserialize");
        assert_eq!(thermostat.unit, "F");

        let expected_thermostat = ThermostatNodeConfig {
            unit: "F".to_string(),
            ..ThermostatNodeConfig::default()
        };
        assert_eq!(thermostat, expected_thermostat);

        let scene: SceneNodeConfig = serde_json::from_str(r#"{"scenes":["scene-a"]}"#)
            .expect("scene partial config must deserialize");
        assert_eq!(
            scene,
            SceneNodeConfig {
                scenes: vec!["scene-a".to_string()],
                ..SceneNodeConfig::default()
            }
        );
    }
}

#[cfg(test)]
mod smarthome_type_serde_tests {
    use super::*;

    #[test]
    fn serializes_and_deserializes_canonical_constants() {
        let types = [
            SmarthomeType::Switch,
            SmarthomeType::Level,
            SmarthomeType::Maintenance,
            SmarthomeType::Contact,
            SmarthomeType::Climate,
            SmarthomeType::Motion,
            SmarthomeType::Button,
            SmarthomeType::Color,
            SmarthomeType::Scene,
            SmarthomeType::Numeric,
            SmarthomeType::Vibration,
            SmarthomeType::Orientation,
            SmarthomeType::WaterSensor,
            SmarthomeType::Shutter,
            SmarthomeType::Tilt,
            SmarthomeType::Thermostat,
            SmarthomeType::Powermeter,
            SmarthomeType::Lock,
            SmarthomeType::Valve,
        ];

        for ty in types {
            let json = serde_json::to_string(&ty).expect("serialize smarthome type");
            assert_eq!(json, format!("\"{}\"", ty.as_str()));

            let parsed: SmarthomeType =
                serde_json::from_str(&json).expect("deserialize smarthome type");
            assert_eq!(parsed, ty);
        }
    }

    #[test]
    fn rejects_non_canonical_short_names() {
        let err = serde_json::from_str::<SmarthomeType>("\"switch\"")
            .expect_err("must reject short name");
        assert!(err.to_string().contains("invalid smarthome type"));
    }
}

#[cfg(test)]
mod tests {
    use rumqttc::{AsyncClient, ClientError};
    use std::{env, time::Duration};
    use tokio::sync::mpsc::channel;

    use homie5::{
        Homie5DeviceProtocol, Homie5Message, HomieDeviceStatus, HomieDomain, HomieID,
        client::{Publish, Subscription},
        device_description::DeviceDescriptionBuilder,
        parse_mqtt_message,
    };

    use crate::{
        SetCommandParser,
        climate_node::{CLIMATE_NODE_DEFAULT_ID, ClimateNodeBuilder},
        level_node::{LEVEL_NODE_DEFAULT_ID, LevelNodeBuilder},
        maintenance_node::{MAINTENANCE_NODE_DEFAULT_ID, MaintenanceNodeBuilder},
        switch_node::{
            SWITCH_NODE_DEFAULT_ID, SwitchNodeActions, SwitchNodeBuilder, SwitchNodeSetEvents,
        },
    };
    #[allow(clippy::large_enum_variant)]
    #[derive(Debug)]
    pub enum ClientEvent {
        Homie(Homie5Message),
        Mqtt(rumqttc::Event),
    }

    struct Settings {
        hostname: String,
        port: u16,
        username: String,
        password: String,
        client_id: String,
        homie_domain: HomieDomain,
    }

    fn get_settings() -> Settings {
        let hostname = env::var("HOMIE_MQTT_HOST").unwrap_or_default();

        let port = if let Ok(port) = env::var("HOMIE_MQTT_PORT") {
            port.parse::<u16>().expect("Not a valid number for port!")
        } else {
            1883
        };

        let username = env::var("HOMIE_MQTT_USERNAME").unwrap_or_default();

        let password = env::var("HOMIE_MQTT_PASSWORD").unwrap_or_default();

        let client_id = if let Ok(client_id) = env::var("HOMIE_MQTT_CLIENT_ID") {
            client_id
        } else {
            String::from("aslkdnlauidhwwkednwek")
        };
        let homie_domain = if let Ok(homie_domain) = env::var("HOMIE_MQTT_HOMIE_DOMAIN") {
            homie_domain.try_into().unwrap_or_default()
        } else {
            HomieDomain::Default
        };

        Settings {
            hostname,
            port,
            username,
            password,
            client_id,
            homie_domain,
        }
    }

    fn qos_to_rumqttc(value: homie5::client::QoS) -> rumqttc::QoS {
        match value {
            homie5::client::QoS::AtLeastOnce => rumqttc::QoS::AtLeastOnce,
            homie5::client::QoS::AtMostOnce => rumqttc::QoS::AtMostOnce,
            homie5::client::QoS::ExactlyOnce => rumqttc::QoS::ExactlyOnce,
        }
    }
    fn lw_to_rumqttc(value: homie5::client::LastWill) -> rumqttc::LastWill {
        rumqttc::LastWill {
            topic: value.topic,
            message: value.message.into(),
            qos: qos_to_rumqttc(value.qos),
            retain: value.retain,
        }
    }

    async fn publish(client: &AsyncClient, p: Publish) -> Result<(), ClientError> {
        client
            .publish(p.topic, qos_to_rumqttc(p.qos), p.retain, p.payload)
            .await
    }

    async fn subscribe(
        client: &AsyncClient,
        subs: impl Iterator<Item = Subscription>,
    ) -> Result<(), ClientError> {
        for sub in subs {
            client.subscribe(sub.topic, qos_to_rumqttc(sub.qos)).await?;
        }
        Ok(())
    }
    #[tokio::test]
    #[ignore]
    async fn test_device() {
        let _settings = get_settings();
        let mut mqttoptions = rumqttc::MqttOptions::new(
            _settings.client_id + "_dev",
            _settings.hostname,
            _settings.port,
        );
        mqttoptions.set_credentials(_settings.username, _settings.password);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        mqttoptions.set_clean_session(true);

        let id: HomieID = "test-hc-smarthome-1".try_into().unwrap();
        let mut switch_state = false;
        let mut switch_state2 = false;
        let mut level_value: i64 = 0;

        let (client, last_will) = Homie5DeviceProtocol::new(id.clone(), _settings.homie_domain);
        mqttoptions.set_last_will(lw_to_rumqttc(last_will));

        let (mqtt_client, mut eventloop) = rumqttc::AsyncClient::new(mqttoptions, 65535);

        let (channel_tx, mut channel_rx) = channel(65535);

        let _handle = tokio::task::spawn(async move {
            loop {
                let event = eventloop.poll().await;

                match event {
                    Ok(event) => {
                        let event = match &event {
                            rumqttc::Event::Incoming(rumqttc::Packet::Publish(p)) => {
                                if let Ok(event) = parse_mqtt_message(&p.topic, &p.payload) {
                                    ClientEvent::Homie(event)
                                } else {
                                    ClientEvent::Mqtt(event)
                                }
                            }
                            _ => ClientEvent::Mqtt(event),
                        };
                        let _ = channel_tx.send(event).await;
                    }

                    Err(err) => {
                        eprintln!("Error received from eventloop: {:#?}", err);
                    }
                }
            }
        });

        let (maintenance_node, maintenance_node_publisher) =
            MaintenanceNodeBuilder::new(Default::default())
                .build_with_publisher(MAINTENANCE_NODE_DEFAULT_ID, &client);

        let (switch_node, switch_node_publisher) = SwitchNodeBuilder::new(&Default::default())
            .build_with_publisher(SWITCH_NODE_DEFAULT_ID, &client);

        let (switch_node2, switch_node_publisher2) = SwitchNodeBuilder::new(&Default::default())
            .build_with_publisher("switch2".try_into().unwrap(), &client);

        let (level_node, level_node_publisher) = LevelNodeBuilder::new(&Default::default())
            .build_with_publisher(LEVEL_NODE_DEFAULT_ID, &client);

        let (climate_node, climate_node_publisher) = ClimateNodeBuilder::new(&Default::default())
            .build_with_publisher(CLIMATE_NODE_DEFAULT_ID, &client);

        let desc = DeviceDescriptionBuilder::new()
            .name("hc-smarthome-test")
            .add_node(MAINTENANCE_NODE_DEFAULT_ID, maintenance_node)
            .add_node(SWITCH_NODE_DEFAULT_ID, switch_node)
            .add_node(LEVEL_NODE_DEFAULT_ID, level_node)
            .add_node(CLIMATE_NODE_DEFAULT_ID, climate_node)
            .add_node("switch2".try_into().unwrap(), switch_node2)
            .build();

        loop {
            let event_opt = channel_rx.recv().await;

            let event = match event_opt {
                Some(event) => event,
                None => {
                    continue;
                }
            };

            match &event {
                ClientEvent::Homie(event) => {
                    if let Homie5Message::PropertySet {
                        property: _,
                        set_value: _,
                    } = event
                    {
                        if let Some(switch_node_event) =
                            switch_node_publisher.parse_set_event(&desc, event).ok()
                        {
                            println!("SwitchNode: {:#?}", switch_node_event);
                            match switch_node_event {
                                SwitchNodeSetEvents::State(swst) => {
                                    switch_state = swst;
                                    let _ = publish(
                                        &mqtt_client,
                                        switch_node_publisher.state_target(swst),
                                    )
                                    .await;

                                    // DO some actual change on a physical device here

                                    let _ =
                                        publish(&mqtt_client, switch_node_publisher.state(swst))
                                            .await;
                                }
                                SwitchNodeSetEvents::Action(action) => {
                                    match action {
                                        SwitchNodeActions::Toggle => {
                                            switch_state = !switch_state;
                                            let _ = publish(
                                                &mqtt_client,
                                                switch_node_publisher.state_target(switch_state),
                                            )
                                            .await;

                                            // DO some actual change on a physical device here

                                            let _ = publish(
                                                &mqtt_client,
                                                switch_node_publisher.state(switch_state),
                                            )
                                            .await;
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(switch_node_event) =
                            switch_node_publisher2.parse_set_event(&desc, event).ok()
                        {
                            println!("SwitchNode2: {:#?}", switch_node_event);
                            match switch_node_event {
                                SwitchNodeSetEvents::State(swst) => {
                                    switch_state2 = swst;
                                    let _ = publish(
                                        &mqtt_client,
                                        switch_node_publisher2.state_target(switch_state2),
                                    )
                                    .await;

                                    // DO some actual change on a physical device here

                                    let _ = publish(
                                        &mqtt_client,
                                        switch_node_publisher2.state(switch_state2),
                                    )
                                    .await;
                                }
                                SwitchNodeSetEvents::Action(action) => {
                                    match action {
                                        SwitchNodeActions::Toggle => {
                                            switch_state2 = !switch_state2;
                                            let _ = publish(
                                                &mqtt_client,
                                                switch_node_publisher2.state_target(switch_state2),
                                            )
                                            .await;

                                            // DO some actual change on a physical device here

                                            let _ = publish(
                                                &mqtt_client,
                                                switch_node_publisher2.state(switch_state2),
                                            )
                                            .await;
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(level_node_event) =
                            level_node_publisher.parse_set_event(&desc, event).ok()
                        {
                            println!("LevelNode: {:#?}", level_node_event);
                            match level_node_event {
                                crate::level_node::LevelNodeSetEvents::Value(value) => {
                                    level_value = value;

                                    let _ = publish(
                                        &mqtt_client,
                                        level_node_publisher.value_target(level_value),
                                    )
                                    .await;

                                    // DO some actual change on a physical device here

                                    let _ = publish(
                                        &mqtt_client,
                                        level_node_publisher.value(level_value),
                                    )
                                    .await;
                                }
                                crate::level_node::LevelNodeSetEvents::Action(action) => {
                                    match action {
                                        crate::level_node::LevelNodeActions::StepUp => {
                                            level_value = std::cmp::min(level_value + 10, 100);
                                        }
                                        crate::level_node::LevelNodeActions::StepDown => {
                                            level_value = std::cmp::max(level_value - 10, 1);
                                        }
                                    }

                                    let _ = publish(
                                        &mqtt_client,
                                        level_node_publisher.value_target(level_value),
                                    )
                                    .await;

                                    // DO some actual change on a physical device here

                                    let _ = publish(
                                        &mqtt_client,
                                        level_node_publisher.value(level_value),
                                    )
                                    .await;
                                }
                            }
                        }
                        println!("Event: {:#?}", event);
                        println!("{}", chrono::Utc::now());
                    }
                }

                ClientEvent::Mqtt(event) => match &event {
                    rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(ca)) => {
                        println!("Connected! Publishing Device: {:#?}", ca);
                        let _ = publish(
                            &mqtt_client,
                            client.publish_state_for_id(&id, HomieDeviceStatus::Init),
                        )
                        .await;
                        let _ = publish(
                            &mqtt_client,
                            client.publish_description_for_id(&id, &desc).unwrap(),
                        )
                        .await;
                        let _ = subscribe(
                            &mqtt_client,
                            client.subscribe_props_for_id(&id, &desc).unwrap(),
                        )
                        .await;
                        let _ = publish(
                            &mqtt_client,
                            maintenance_node_publisher.low_battery(false).unwrap(),
                        )
                        .await;
                        let _ = publish(
                            &mqtt_client,
                            maintenance_node_publisher
                                .last_update(chrono::Utc::now())
                                .unwrap(),
                        )
                        .await;
                        let _ = publish(
                            &mqtt_client,
                            maintenance_node_publisher.reachable(true).unwrap(),
                        )
                        .await;
                        let _ = publish(
                            &mqtt_client,
                            switch_node_publisher.state_target(switch_state),
                        )
                        .await;
                        let _ =
                            publish(&mqtt_client, switch_node_publisher.state(switch_state)).await;
                        let _ = publish(
                            &mqtt_client,
                            switch_node_publisher2.state_target(switch_state2),
                        )
                        .await;
                        let _ = publish(&mqtt_client, switch_node_publisher2.state(switch_state2))
                            .await;
                        let _ =
                            publish(&mqtt_client, level_node_publisher.value(level_value)).await;
                        let _ =
                            publish(&mqtt_client, climate_node_publisher.temperature(12.4)).await;
                        let _ = publish(&mqtt_client, climate_node_publisher.humidity(64)).await;

                        let _ = publish(
                            &mqtt_client,
                            client.publish_state_for_id(&id, HomieDeviceStatus::Ready),
                        )
                        .await;
                    }
                    rumqttc::Event::Incoming(rumqttc::Packet::Publish(p)) => {
                        println!("MQTT Publish: {:#?}", p);
                    }
                    _ => {}
                },
            }
        }
    }
}

pub mod alerts;
pub mod button_node;
pub mod colorlight_node;
pub mod contact_node;
pub mod dimmer_node;
pub mod light_scene_node;
pub mod maintenance_node;
pub mod motion_node;
pub mod numeric_sensor_node;
pub mod orientation_node;
pub mod powermeter_node;
pub mod shutter_node;
pub mod switch_node;
pub mod thermostat_node;
pub mod tilt_node;
pub mod vibration_node;
pub mod water_sensor_node;
pub mod weather_node;

use std::{fmt, str::FromStr};

use button_node::ButtonNodeConfig;
use colorlight_node::{ColorlightNode, ColorlightNodeConfig};
use contact_node::ContactNode;
use dimmer_node::{DimmerNode, DimmerNodeConfig};
use light_scene_node::LightSceneNodeConfig;
use maintenance_node::{MaintenanceNode, MaintenanceNodeConfig};
use motion_node::{MotionNode, MotionNodeConfig};
use numeric_sensor_node::NumericSensorNode;
use powermeter_node::{PowermeterNode, PowermeterNodeConfig};
use serde::{Deserialize, Serialize};
use shutter_node::{ShutterNode, ShutterNodeConfig};
use switch_node::{SwitchNode, SwitchNodeConfig};
use thermostat_node::ThermostatNodeConfig;
use tilt_node::TiltNode;
use vibration_node::VibrationNodeConfig;
use water_sensor_node::WaterSensorNode;
use weather_node::{WeatherNode, WeatherNodeConfig};

/// Helper macro to generate static smarthome type strings
macro_rules! create_smarthome_type {
    ($type:expr) => {
        concat!("homie-homecontrol/v1/type=", $type)
    };
}

/// Helper macro to generate static smarthome type strings for extensions
#[macro_export]
macro_rules! create_smarthome_type_extension {
    ($type:expr) => {
        concat!("homie-homecontrol/v1/extension/type=", $type)
    };
}

pub const SMARTHOME_NS_V1: &str = "homie-homecontrol/v1";

pub const SMARTHOME_TYPE_MAINTENANCE: &str = create_smarthome_type!("maintenance");
pub const SMARTHOME_TYPE_SWITCH: &str = create_smarthome_type!("switch");
pub const SMARTHOME_TYPE_DIMMER: &str = create_smarthome_type!("dimmer");
pub const SMARTHOME_TYPE_CONTACT: &str = create_smarthome_type!("contact");
pub const SMARTHOME_TYPE_WEATHER: &str = create_smarthome_type!("weather");
pub const SMARTHOME_TYPE_MOTION: &str = create_smarthome_type!("motion");
pub const SMARTHOME_TYPE_BUTTON: &str = create_smarthome_type!("button");
pub const SMARTHOME_TYPE_COLORLIGHT: &str = create_smarthome_type!("colorlight");
pub const SMARTHOME_TYPE_LIGHTSCENE: &str = create_smarthome_type!("lightscene");
pub const SMARTHOME_TYPE_NUMERIC: &str = create_smarthome_type!("numeric");
pub const SMARTHOME_TYPE_VIBRATION: &str = create_smarthome_type!("vibration");
pub const SMARTHOME_TYPE_ORIENTATION: &str = create_smarthome_type!("orientation");
pub const SMARTHOME_TYPE_WATER_SENSOR: &str = create_smarthome_type!("water");
pub const SMARTHOME_TYPE_SHUTTER: &str = create_smarthome_type!("shutter");
pub const SMARTHOME_TYPE_TILT: &str = create_smarthome_type!("tilt");
pub const SMARTHOME_TYPE_THERMOSTAT: &str = create_smarthome_type!("thermostat");
pub const SMARTHOME_TYPE_POWERMETER: &str = create_smarthome_type!("powermeter");

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
    pub fn new(property_id: impl Into<String>, payload: impl Into<String>, kind: ParseErrorKind) -> Self {
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

#[cfg(test)]
mod parse_outcome_tests {
    use super::*;

    #[test]
    fn parse_outcome_ok_returns_only_parsed_value() {
        assert_eq!(ParseOutcome::Parsed(42).ok(), Some(42));
        assert_eq!(ParseOutcome::<i32>::NoMatch.ok(), None);
        assert_eq!(
            ParseOutcome::<i32>::Invalid(ParseError::new("x", "y", ParseErrorKind::InvalidHomieValue)).ok(),
            None
        );
    }

    #[test]
    fn parse_outcome_into_result_preserves_invalid_error() {
        let no_match = ParseOutcome::<i32>::NoMatch.into_result().expect("no match should be ok");
        assert_eq!(no_match, None);

        let parsed = ParseOutcome::Parsed(7).into_result().expect("parsed should be ok");
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
        let switch: SwitchNodeConfig = serde_json::from_str("{}").expect("switch config must deserialize");
        assert_eq!(switch, SwitchNodeConfig::default());

        let dimmer: DimmerNodeConfig = serde_json::from_str("{}").expect("dimmer config must deserialize");
        assert_eq!(dimmer, DimmerNodeConfig::default());

        let shutter: ShutterNodeConfig = serde_json::from_str("{}").expect("shutter config must deserialize");
        assert_eq!(shutter, ShutterNodeConfig::default());

        let colorlight: ColorlightNodeConfig = serde_json::from_str("{}").expect("colorlight config must deserialize");
        assert_eq!(colorlight, ColorlightNodeConfig::default());

        let light_scene: LightSceneNodeConfig = serde_json::from_str("{}").expect("light scene config must deserialize");
        assert_eq!(light_scene, LightSceneNodeConfig::default());

        let thermostat: ThermostatNodeConfig = serde_json::from_str("{}").expect("thermostat config must deserialize");
        assert_eq!(thermostat, ThermostatNodeConfig::default());

        let weather: WeatherNodeConfig = serde_json::from_str("{}").expect("weather config must deserialize");
        assert_eq!(weather, WeatherNodeConfig::default());

        let motion: MotionNodeConfig = serde_json::from_str("{}").expect("motion config must deserialize");
        assert_eq!(motion, MotionNodeConfig::default());

        let vibration: VibrationNodeConfig = serde_json::from_str("{}").expect("vibration config must deserialize");
        assert_eq!(vibration, VibrationNodeConfig::default());

        let maintenance: MaintenanceNodeConfig =
            serde_json::from_str("{}").expect("maintenance config must deserialize");
        assert_eq!(maintenance, MaintenanceNodeConfig::default());

        let button: ButtonNodeConfig = serde_json::from_str("{}").expect("button config must deserialize");
        assert_eq!(button, ButtonNodeConfig::default());

        let powermeter: PowermeterNodeConfig = serde_json::from_str("{}").expect("powermeter config must deserialize");
        assert_eq!(powermeter, PowermeterNodeConfig::default());
    }

    #[test]
    fn partial_config_deserialization_keeps_defaults_for_missing_fields() {
        let thermostat: ThermostatNodeConfig =
            serde_json::from_str(r#"{"unit":"F"}"#).expect("thermostat partial config must deserialize");
        assert_eq!(thermostat.unit, "F");

        let expected_thermostat = ThermostatNodeConfig {
            unit: "F".to_string(),
            ..ThermostatNodeConfig::default()
        };
        assert_eq!(thermostat, expected_thermostat);

        let light_scene: LightSceneNodeConfig =
            serde_json::from_str(r#"{"scenes":["scene-a"]}"#).expect("light scene partial config must deserialize");
        assert_eq!(
            light_scene,
            LightSceneNodeConfig {
                scenes: vec!["scene-a".to_string()],
                ..LightSceneNodeConfig::default()
            }
        );
    }
}

/// SmarthomeType enum representing various smart home device types.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SmarthomeType {
    Switch,
    Dimmer,
    Maintenance,
    Contact,
    Weather,
    Motion,
    Button,
    ColorLight,
    LightScene,
    Numeric,
    Vibration,
    Orientation,
    WaterSensor,
    Shutter,
    Tilt,
    Thermostat,
    Powermeter,
}

impl SmarthomeType {
    /// Convert the enum variant into its corresponding string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            SmarthomeType::Switch => SMARTHOME_TYPE_SWITCH,
            SmarthomeType::Dimmer => SMARTHOME_TYPE_DIMMER,
            SmarthomeType::Maintenance => SMARTHOME_TYPE_MAINTENANCE,
            SmarthomeType::Contact => SMARTHOME_TYPE_CONTACT,
            SmarthomeType::Weather => SMARTHOME_TYPE_WEATHER,
            SmarthomeType::Motion => SMARTHOME_TYPE_MOTION,
            SmarthomeType::Button => SMARTHOME_TYPE_BUTTON,
            SmarthomeType::ColorLight => SMARTHOME_TYPE_COLORLIGHT,
            SmarthomeType::LightScene => SMARTHOME_TYPE_LIGHTSCENE,
            SmarthomeType::Numeric => SMARTHOME_TYPE_NUMERIC,
            SmarthomeType::Vibration => SMARTHOME_TYPE_VIBRATION,
            SmarthomeType::Orientation => SMARTHOME_TYPE_ORIENTATION,
            SmarthomeType::WaterSensor => SMARTHOME_TYPE_WATER_SENSOR,
            SmarthomeType::Shutter => SMARTHOME_TYPE_SHUTTER,
            SmarthomeType::Tilt => SMARTHOME_TYPE_TILT,
            SmarthomeType::Thermostat => SMARTHOME_TYPE_THERMOSTAT,
            SmarthomeType::Powermeter => SMARTHOME_TYPE_POWERMETER,
        }
    }

    /// Create a SmarthomeType from a string containing a constant value.
    pub fn from_constant(value: &str) -> Option<Self> {
        match value {
            SMARTHOME_TYPE_SWITCH => Some(SmarthomeType::Switch),
            SMARTHOME_TYPE_DIMMER => Some(SmarthomeType::Dimmer),
            SMARTHOME_TYPE_MAINTENANCE => Some(SmarthomeType::Maintenance),
            SMARTHOME_TYPE_CONTACT => Some(SmarthomeType::Contact),
            SMARTHOME_TYPE_WEATHER => Some(SmarthomeType::Weather),
            SMARTHOME_TYPE_MOTION => Some(SmarthomeType::Motion),
            SMARTHOME_TYPE_BUTTON => Some(SmarthomeType::Button),
            SMARTHOME_TYPE_COLORLIGHT => Some(SmarthomeType::ColorLight),
            SMARTHOME_TYPE_LIGHTSCENE => Some(SmarthomeType::LightScene),
            SMARTHOME_TYPE_NUMERIC => Some(SmarthomeType::Numeric),
            SMARTHOME_TYPE_VIBRATION => Some(SmarthomeType::Vibration),
            SMARTHOME_TYPE_ORIENTATION => Some(SmarthomeType::Orientation),
            SMARTHOME_TYPE_WATER_SENSOR => Some(SmarthomeType::WaterSensor),
            SMARTHOME_TYPE_SHUTTER => Some(SmarthomeType::Shutter),
            SMARTHOME_TYPE_TILT => Some(SmarthomeType::Tilt),
            SMARTHOME_TYPE_THERMOSTAT => Some(SmarthomeType::Thermostat),
            SMARTHOME_TYPE_POWERMETER => Some(SmarthomeType::Powermeter),
            _ => None,
        }
    }
}

/// Implement `FromStr` to parse a SmarthomeType from a string.
impl FromStr for SmarthomeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SmarthomeType::from_constant(s).ok_or(())
    }
}

/// Implement `Display` for better formatting and output.
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

#[cfg(test)]
mod smarthome_type_serde_tests {
    use super::*;

    #[test]
    fn serializes_and_deserializes_canonical_constants() {
        let types = [
            SmarthomeType::Switch,
            SmarthomeType::Dimmer,
            SmarthomeType::Maintenance,
            SmarthomeType::Contact,
            SmarthomeType::Weather,
            SmarthomeType::Motion,
            SmarthomeType::Button,
            SmarthomeType::ColorLight,
            SmarthomeType::LightScene,
            SmarthomeType::Numeric,
            SmarthomeType::Vibration,
            SmarthomeType::Orientation,
            SmarthomeType::WaterSensor,
            SmarthomeType::Shutter,
            SmarthomeType::Tilt,
            SmarthomeType::Thermostat,
            SmarthomeType::Powermeter,
        ];

        for ty in types {
            let json = serde_json::to_string(&ty).expect("serialize smarthome type");
            assert_eq!(json, format!("\"{}\"", ty.as_str()));

            let parsed: SmarthomeType = serde_json::from_str(&json).expect("deserialize smarthome type");
            assert_eq!(parsed, ty);
        }
    }

    #[test]
    fn rejects_non_canonical_short_names() {
        let err = serde_json::from_str::<SmarthomeType>("\"switch\"").expect_err("must reject short name");
        assert!(err.to_string().contains("invalid smarthome type"));
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SmarthomeProperyConfig {
    Button(ButtonNodeConfig),
    ColorLight(ColorlightNodeConfig),
    Dimmer(DimmerNodeConfig),
    LightScene(LightSceneNodeConfig),
    Maintenance(MaintenanceNodeConfig),
    Motion(MotionNodeConfig),
    Shutter(ShutterNodeConfig),
    Switch(SwitchNodeConfig),
    Thermostat(ThermostatNodeConfig),
    Vibration(VibrationNodeConfig),
    Weather(WeatherNodeConfig),
    Powermeter(PowermeterNodeConfig),
}

#[derive(Debug)]
pub enum SmarthomeNode {
    MaintenanceNode(MaintenanceNode),
    SwitchNode(SwitchNode),
    DimmerNode(DimmerNode),
    WeatherNode(WeatherNode),
    ContactNode(ContactNode),
    MotionNode(MotionNode),
    ColorlightNode(ColorlightNode),
    NumericSensorNode(NumericSensorNode),
    WaterSensor(WaterSensorNode),
    ShutterNode(ShutterNode),
    TiltNode(TiltNode),
    Powermeter(PowermeterNode),
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
        dimmer_node::{DIMMER_NODE_DEFAULT_ID, DimmerNodeBuilder},
        maintenance_node::{MAINTENANCE_NODE_DEFAULT_ID, MaintenanceNodeBuilder},
        switch_node::{
            SWITCH_NODE_DEFAULT_ID, SwitchNodeActions, SwitchNodeBuilder, SwitchNodeSetEvents,
        },
        SetCommandParser,
        weather_node::{WEATHER_NODE_DEFAULT_ID, WeatherNodeBuilder},
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
        let mut dimmer_state: i64 = 0;

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
                .build_with_publisher(MAINTENANCE_NODE_DEFAULT_ID.try_into().unwrap(), &client);

        let (switch_node, switch_node_publisher) = SwitchNodeBuilder::new(&Default::default())
            .build_with_publisher(SWITCH_NODE_DEFAULT_ID.try_into().unwrap(), &client);

        let (switch_node2, switch_node_publisher2) = SwitchNodeBuilder::new(&Default::default())
            .build_with_publisher("switch2".try_into().unwrap(), &client);

        let (dimmer_node, dimmer_node_publisher) = DimmerNodeBuilder::new(&Default::default())
            .build_with_publisher(DIMMER_NODE_DEFAULT_ID.try_into().unwrap(), &client);

        let (weather_node, weather_node_publisher) = WeatherNodeBuilder::new(&Default::default())
            .build_with_publisher(WEATHER_NODE_DEFAULT_ID.try_into().unwrap(), &client);

        let desc = DeviceDescriptionBuilder::new()
            .name("hc-smarthome-test")
            .add_node(
                MAINTENANCE_NODE_DEFAULT_ID.try_into().unwrap(),
                maintenance_node,
            )
            .add_node(SWITCH_NODE_DEFAULT_ID.try_into().unwrap(), switch_node)
            .add_node(DIMMER_NODE_DEFAULT_ID.try_into().unwrap(), dimmer_node)
            .add_node(WEATHER_NODE_DEFAULT_ID.try_into().unwrap(), weather_node)
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
                        if let Some(dimmer_node_event) =
                            dimmer_node_publisher.parse_set_event(&desc, event).ok()
                        {
                            println!("DimmerNode: {:#?}", dimmer_node_event);
                            match dimmer_node_event {
                                crate::dimmer_node::DimmerNodeSetEvents::Brightness(value) => {
                                    dimmer_state = value;

                                    let _ = publish(
                                        &mqtt_client,
                                        dimmer_node_publisher.brightness_target(dimmer_state),
                                    )
                                    .await;

                                    // DO some actual change on a physical device here

                                    let _ = publish(
                                        &mqtt_client,
                                        dimmer_node_publisher.brightness(dimmer_state),
                                    )
                                    .await;
                                }
                                crate::dimmer_node::DimmerNodeSetEvents::Action(action) => {
                                    match action {
                                        crate::dimmer_node::DimmerNodeActions::Brighter => {
                                            dimmer_state = std::cmp::min(dimmer_state + 10, 100);
                                        }
                                        crate::dimmer_node::DimmerNodeActions::Darker => {
                                            dimmer_state = std::cmp::max(dimmer_state - 10, 1);
                                        }
                                    }

                                    let _ = publish(
                                        &mqtt_client,
                                        dimmer_node_publisher.brightness_target(dimmer_state),
                                    )
                                    .await;

                                    // DO some actual change on a physical device here

                                    let _ = publish(
                                        &mqtt_client,
                                        dimmer_node_publisher.brightness(dimmer_state),
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
                            publish(&mqtt_client, dimmer_node_publisher.brightness(dimmer_state))
                                .await;
                        let _ =
                            publish(&mqtt_client, weather_node_publisher.temperature(12.4)).await;
                        let _ = publish(&mqtt_client, weather_node_publisher.humidity(64)).await;

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

use chrono::prelude::*;
use homie5::{
    HOMIE_UNIT_DEGREE_CELSIUS, HOMIE_UNIT_PERCENT, HOMIE_UNIT_SECONDS, Homie5DeviceProtocol,
    Homie5Message, Homie5ProtocolError, HomieID, HomieValue, NodeRef, PropertyRef,
    device_description::{
        FloatRange, HomieDeviceDescription, HomieNodeDescription, IntegerRange,
        NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, PropSpec, SMARTHOME_CAP_THERMOSTAT, SetCommandParser,
};

pub const THERMOSTAT_NODE_DEFAULT_ID: HomieID = HomieID::new_const("thermostat");
pub const THERMOSTAT_NODE_DEFAULT_NAME: &str = "Thermostat";

pub const THERMOSTAT_NODE_SET_TEMPERATURE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("set-temperature"),
    retained: true,
    settable: true,
};
pub const THERMOSTAT_NODE_VALVE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("valve"),
    retained: true,
    settable: false,
};
/// Base operating mode. It never carries a temporary overlay such as boost or party; a running
/// boost is reported through `boost-state`, `boost-end` and `boost-duration` instead.
pub const THERMOSTAT_NODE_MODE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("mode"),
    retained: true,
    settable: true,
};
pub const THERMOSTAT_NODE_WINDOWOPEN_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("window-open"),
    retained: true,
    settable: false,
};
/// Boost overlay on top of the base `mode`. Settable so automations can start or stop a boost.
pub const THERMOSTAT_NODE_BOOST_STATE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("boost-state"),
    retained: true,
    settable: true,
};
/// Time at which the running boost ends. Meaningful only while `boost-state` is true. Homie allows
/// empty payloads only for strings, so the last end time stays published after a boost ends.
pub const THERMOSTAT_NODE_BOOST_END_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("boost-end"),
    retained: true,
    settable: false,
};
/// Configured boost length in seconds, from the device setting.
pub const THERMOSTAT_NODE_BOOST_DURATION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("boost-duration"),
    retained: true,
    settable: false,
};

#[derive(Debug)]
pub struct ThermostatNode {
    pub publisher: ThermostatNodePublisher,
    pub set_temperature: f64,
    pub set_temperature_target: f64,
    pub valve: Option<i64>,
    pub mode: Option<ThermostatNodeModes>,
    pub windowopen: Option<bool>,
    pub boost_state: Option<bool>,
    pub boost_end: Option<DateTime<Utc>>,
    pub boost_duration: Option<i64>,
}

#[derive(Debug, Default, Copy, PartialEq, Clone, Serialize, Deserialize)]
pub enum ThermostatNodeModes {
    #[default]
    Off,
    Auto,
    Manual,
    /// Deprecated for publishing: `mode` is the base mode, never a temporary overlay.
    /// Still parsed for compatibility with older publishers.
    Party,
    /// Deprecated for publishing: report a boost through `boost-state`/`boost-end` instead.
    /// Still parsed for compatibility with older publishers.
    Boost,
    Cool,
    Heat,
    EmergencyHeating,
    Precooling,
    FanOnly,
    Dry,
    Sleep,
}

impl ThermostatNodeModes {
    fn as_str(&self) -> &'static str {
        match self {
            ThermostatNodeModes::Off => "off",
            ThermostatNodeModes::Auto => "auto",
            ThermostatNodeModes::Manual => "manual",
            ThermostatNodeModes::Party => "party",
            ThermostatNodeModes::Boost => "boost",
            ThermostatNodeModes::Cool => "cool",
            ThermostatNodeModes::Heat => "heat",
            ThermostatNodeModes::EmergencyHeating => "emergency-heating",
            ThermostatNodeModes::Precooling => "precooling",
            ThermostatNodeModes::FanOnly => "fan-only",
            ThermostatNodeModes::Dry => "dry",
            ThermostatNodeModes::Sleep => "sleep",
        }
    }
}

impl From<&ThermostatNodeModes> for String {
    fn from(value: &ThermostatNodeModes) -> Self {
        value.as_str().to_string()
    }
}

impl From<&ThermostatNodeModes> for &'static str {
    fn from(value: &ThermostatNodeModes) -> Self {
        value.as_str()
    }
}

impl TryFrom<String> for ThermostatNodeModes {
    type Error = Homie5ProtocolError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.try_into()
    }
}

impl TryFrom<&str> for ThermostatNodeModes {
    type Error = Homie5ProtocolError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "off" => Ok(ThermostatNodeModes::Off),
            "auto" => Ok(ThermostatNodeModes::Auto),
            "manual" => Ok(ThermostatNodeModes::Manual),
            "party" => Ok(ThermostatNodeModes::Party),
            "boost" => Ok(ThermostatNodeModes::Boost),
            "cool" => Ok(ThermostatNodeModes::Cool),
            "heat" => Ok(ThermostatNodeModes::Heat),
            "emergency-heating" => Ok(ThermostatNodeModes::EmergencyHeating),
            "precooling" => Ok(ThermostatNodeModes::Precooling),
            "fan-only" => Ok(ThermostatNodeModes::FanOnly),
            "dry" => Ok(ThermostatNodeModes::Dry),
            "sleep" => Ok(ThermostatNodeModes::Sleep),
            _ => Err(Homie5ProtocolError::InvalidPayload),
        }
    }
}

#[derive(Debug)]
pub enum ThermostatNodeSetEvents {
    Mode(ThermostatNodeModes),
    SetTemperature(f64),
    Boost(bool),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThermostatNodeConfig {
    pub unit: String,
    pub valve: bool,
    pub windowopen: bool,
    pub boost_state: bool,
    pub boost_end: bool,
    pub boost_duration: bool,
    pub mode: bool,
    pub modes: Vec<ThermostatNodeModes>,
    pub temp_range: FloatRange,
}

impl Default for ThermostatNodeConfig {
    fn default() -> Self {
        Self {
            unit: HOMIE_UNIT_DEGREE_CELSIUS.to_string(),
            valve: true,
            windowopen: true,
            boost_state: true,
            boost_end: true,
            boost_duration: true,
            mode: true,
            modes: vec![ThermostatNodeModes::Auto, ThermostatNodeModes::Manual],
            temp_range: FloatRange {
                min: Some(5.0),
                max: Some(32.0),
                step: Some(0.5),
            },
        }
    }
}

pub struct ThermostatNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl ThermostatNodeBuilder {
    pub fn new(config: &ThermostatNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(THERMOSTAT_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_THERMOSTAT);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &ThermostatNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            THERMOSTAT_NODE_SET_TEMPERATURE_PROP.id,
            PropertyDescriptionBuilder::float()
                .name("Set target temperature")
                .float_range(config.temp_range.clone())
                .unit(config.unit.to_owned())
                .settable(THERMOSTAT_NODE_SET_TEMPERATURE_PROP.settable)
                .retained(THERMOSTAT_NODE_SET_TEMPERATURE_PROP.retained)
                .build(),
        )
        .add_property_cond(THERMOSTAT_NODE_VALVE_PROP.id, config.valve, || {
            PropertyDescriptionBuilder::integer()
                .name("Valve opening Level")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: Some(100),
                    step: None,
                })
                .unit(HOMIE_UNIT_PERCENT)
                .settable(THERMOSTAT_NODE_VALVE_PROP.settable)
                .retained(THERMOSTAT_NODE_VALVE_PROP.retained)
                .build()
        })
        .add_property_cond(
            THERMOSTAT_NODE_WINDOWOPEN_PROP.id,
            config.windowopen,
            || {
                PropertyDescriptionBuilder::boolean()
                    .name("Window open detected")
                    .boolean_labels("closed", "open")
                    .settable(THERMOSTAT_NODE_WINDOWOPEN_PROP.settable)
                    .retained(THERMOSTAT_NODE_WINDOWOPEN_PROP.retained)
                    .build()
            },
        )
        .add_property_cond(
            THERMOSTAT_NODE_BOOST_STATE_PROP.id,
            config.boost_state,
            || {
                PropertyDescriptionBuilder::boolean()
                    .name("Boost mode active")
                    .settable(THERMOSTAT_NODE_BOOST_STATE_PROP.settable)
                    .retained(THERMOSTAT_NODE_BOOST_STATE_PROP.retained)
                    .build()
            },
        )
        .add_property_cond(THERMOSTAT_NODE_BOOST_END_PROP.id, config.boost_end, || {
            PropertyDescriptionBuilder::datetime()
                .name("Boost end")
                .settable(THERMOSTAT_NODE_BOOST_END_PROP.settable)
                .retained(THERMOSTAT_NODE_BOOST_END_PROP.retained)
                .build()
        })
        .add_property_cond(
            THERMOSTAT_NODE_BOOST_DURATION_PROP.id,
            config.boost_duration,
            || {
                PropertyDescriptionBuilder::integer()
                    .name("Boost duration")
                    .integer_range(IntegerRange {
                        min: Some(0),
                        max: None,
                        step: None,
                    })
                    .unit(HOMIE_UNIT_SECONDS)
                    .settable(THERMOSTAT_NODE_BOOST_DURATION_PROP.settable)
                    .retained(THERMOSTAT_NODE_BOOST_DURATION_PROP.retained)
                    .build()
            },
        )
        .add_property_cond(THERMOSTAT_NODE_MODE_PROP.id, config.mode, || {
            PropertyDescriptionBuilder::enumeration(config.modes.iter().map(<&str>::from))
                .unwrap()
                .name("Mode")
                .settable(THERMOSTAT_NODE_MODE_PROP.settable)
                .retained(THERMOSTAT_NODE_MODE_PROP.retained)
                .build()
        })
    }

    pub fn name<S: Into<String>>(mut self, name: impl Into<Option<S>>) -> Self {
        self.node_builder = self.node_builder.name(name);
        self
    }

    pub fn build(self) -> HomieNodeDescription {
        self.node_builder.build()
    }

    pub fn build_with_publisher(
        self,
        node_id: HomieID,
        client: &Homie5DeviceProtocol,
    ) -> (HomieNodeDescription, ThermostatNodePublisher) {
        let did = client.id().clone();
        (
            self.node_builder.build(),
            ThermostatNodePublisher::new(
                NodeRef::new(client.homie_domain().to_owned(), did, node_id),
                client.clone(),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use homie5::{HomieDataType, device_description::HomiePropertyFormat};

    fn base_disabled_config() -> ThermostatNodeConfig {
        ThermostatNodeConfig {
            unit: "F".to_string(),
            valve: false,
            windowopen: false,
            boost_state: false,
            boost_end: false,
            boost_duration: false,
            mode: false,
            modes: vec![ThermostatNodeModes::Cool, ThermostatNodeModes::Heat],
            temp_range: FloatRange {
                min: Some(10.0),
                max: Some(30.0),
                step: Some(1.0),
            },
        }
    }

    #[test]
    fn set_temperature_respects_unit_and_temp_range_config() {
        let config = base_disabled_config();
        let node = ThermostatNodeBuilder::new(&config).build();

        let set_temperature = node
            .properties
            .get(&THERMOSTAT_NODE_SET_TEMPERATURE_PROP.id)
            .expect("set-temperature property must exist");

        assert_eq!(set_temperature.unit.as_deref(), Some("F"));
        assert_eq!(
            set_temperature.format,
            HomiePropertyFormat::FloatRange(FloatRange {
                min: Some(10.0),
                max: Some(30.0),
                step: Some(1.0),
            })
        );
    }

    #[test]
    fn each_boolean_config_gates_the_expected_optional_property() {
        let gated_properties = [
            THERMOSTAT_NODE_VALVE_PROP.id,
            THERMOSTAT_NODE_WINDOWOPEN_PROP.id,
            THERMOSTAT_NODE_BOOST_STATE_PROP.id,
            THERMOSTAT_NODE_BOOST_END_PROP.id,
            THERMOSTAT_NODE_BOOST_DURATION_PROP.id,
            THERMOSTAT_NODE_MODE_PROP.id,
        ];

        for expected_property_id in gated_properties.iter() {
            let mut config = base_disabled_config();
            if expected_property_id == &THERMOSTAT_NODE_VALVE_PROP.id {
                config.valve = true;
            } else if expected_property_id == &THERMOSTAT_NODE_WINDOWOPEN_PROP.id {
                config.windowopen = true;
            } else if expected_property_id == &THERMOSTAT_NODE_BOOST_STATE_PROP.id {
                config.boost_state = true;
            } else if expected_property_id == &THERMOSTAT_NODE_BOOST_END_PROP.id {
                config.boost_end = true;
            } else if expected_property_id == &THERMOSTAT_NODE_BOOST_DURATION_PROP.id {
                config.boost_duration = true;
            } else if expected_property_id == &THERMOSTAT_NODE_MODE_PROP.id {
                config.mode = true;
            } else {
                unreachable!("unknown thermostat property id");
            }

            let node = ThermostatNodeBuilder::new(&config).build();

            assert!(
                node.properties
                    .contains_key(&THERMOSTAT_NODE_SET_TEMPERATURE_PROP.id),
                "set-temperature property must always exist"
            );

            for property_id in gated_properties.iter() {
                let expected_present = property_id == expected_property_id;
                assert_eq!(
                    node.properties.contains_key(property_id),
                    expected_present,
                    "property {property_id} presence mismatch"
                );
            }
        }
    }

    #[test]
    fn mode_property_uses_configured_modes() {
        let mut config = base_disabled_config();
        config.mode = true;

        let node = ThermostatNodeBuilder::new(&config).build();
        let mode_property = node
            .properties
            .get(&THERMOSTAT_NODE_MODE_PROP.id)
            .expect("mode property must exist when mode config is enabled");

        assert_eq!(
            mode_property.format,
            HomiePropertyFormat::Enum(vec!["cool".to_string(), "heat".to_string()])
        );
    }

    #[test]
    fn boost_end_is_a_retained_read_only_datetime() {
        let node = ThermostatNodeBuilder::new(&ThermostatNodeConfig::default()).build();
        let boost_end = node
            .properties
            .get(&THERMOSTAT_NODE_BOOST_END_PROP.id)
            .expect("boost-end property must exist by default");

        assert_eq!(boost_end.datatype, HomieDataType::Datetime);
        assert_eq!(boost_end.format, HomiePropertyFormat::Empty);
        assert_eq!(boost_end.unit, None);
        assert!(boost_end.retained);
        assert!(!boost_end.settable);
    }

    #[test]
    fn boost_duration_is_a_retained_read_only_seconds_integer() {
        let node = ThermostatNodeBuilder::new(&ThermostatNodeConfig::default()).build();
        let boost_duration = node
            .properties
            .get(&THERMOSTAT_NODE_BOOST_DURATION_PROP.id)
            .expect("boost-duration property must exist by default");

        assert_eq!(boost_duration.datatype, HomieDataType::Integer);
        assert_eq!(
            boost_duration.format,
            HomiePropertyFormat::IntegerRange(IntegerRange {
                min: Some(0),
                max: None,
                step: None,
            })
        );
        assert_eq!(boost_duration.unit.as_deref(), Some(HOMIE_UNIT_SECONDS));
        assert!(boost_duration.retained);
        assert!(!boost_duration.settable);
    }

    #[test]
    fn boost_state_stays_settable() {
        let node = ThermostatNodeBuilder::new(&ThermostatNodeConfig::default()).build();
        let boost_state = node
            .properties
            .get(&THERMOSTAT_NODE_BOOST_STATE_PROP.id)
            .expect("boost-state property must exist by default");

        assert!(boost_state.settable);
        assert!(boost_state.retained);
    }

    #[test]
    fn config_without_boost_fields_defaults_new_flags_to_true() {
        let config: ThermostatNodeConfig = serde_json::from_str(r#"{"valve": false}"#).unwrap();

        assert!(config.boost_end);
        assert!(config.boost_duration);
    }
}

#[derive(Debug)]
pub struct ThermostatNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
}

impl ThermostatNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn set_temperature(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &THERMOSTAT_NODE_SET_TEMPERATURE_PROP.id,
            value.to_string(),
            THERMOSTAT_NODE_SET_TEMPERATURE_PROP.retained,
        )
    }

    pub fn set_temperature_target(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &THERMOSTAT_NODE_SET_TEMPERATURE_PROP.id,
            value.to_string(),
            THERMOSTAT_NODE_SET_TEMPERATURE_PROP.retained,
        )
    }

    pub fn mode(&self, mode: ThermostatNodeModes) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &THERMOSTAT_NODE_MODE_PROP.id,
            &mode,
            THERMOSTAT_NODE_MODE_PROP.retained,
        )
    }

    pub fn mode_target(&self, mode: ThermostatNodeModes) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &THERMOSTAT_NODE_MODE_PROP.id,
            &mode,
            THERMOSTAT_NODE_MODE_PROP.retained,
        )
    }

    pub fn boost(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &THERMOSTAT_NODE_BOOST_STATE_PROP.id,
            value.to_string(),
            THERMOSTAT_NODE_BOOST_STATE_PROP.retained,
        )
    }

    pub fn boost_end(&self, value: DateTime<Utc>) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &THERMOSTAT_NODE_BOOST_END_PROP.id,
            HomieValue::DateTime(value),
            THERMOSTAT_NODE_BOOST_END_PROP.retained,
        )
    }

    pub fn boost_duration(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &THERMOSTAT_NODE_BOOST_DURATION_PROP.id,
            value.to_string(),
            THERMOSTAT_NODE_BOOST_DURATION_PROP.retained,
        )
    }

    pub fn valve(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &THERMOSTAT_NODE_VALVE_PROP.id,
            value.to_string(),
            THERMOSTAT_NODE_VALVE_PROP.retained,
        )
    }

    pub fn windowopen(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &THERMOSTAT_NODE_WINDOWOPEN_PROP.id,
            value.to_string(),
            THERMOSTAT_NODE_WINDOWOPEN_PROP.retained,
        )
    }
}

impl SetCommandParser for ThermostatNodePublisher {
    type Event = ThermostatNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &THERMOSTAT_NODE_SET_TEMPERATURE_PROP.id) {
            let Some(parsed) = desc.with_property(property, |prop_desc| {
                HomieValue::parse(set_value, prop_desc)
            }) else {
                return ParseOutcome::Invalid(ParseError::new(
                    property_id,
                    set_value,
                    ParseErrorKind::MissingPropertyDescription,
                ));
            };

            match parsed {
                Ok(HomieValue::Float(value)) => {
                    ParseOutcome::Parsed(ThermostatNodeSetEvents::SetTemperature(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &THERMOSTAT_NODE_MODE_PROP.id) {
            let Some(parsed) = desc.with_property(property, |prop_desc| {
                HomieValue::parse(set_value, prop_desc)
            }) else {
                return ParseOutcome::Invalid(ParseError::new(
                    property_id,
                    set_value,
                    ParseErrorKind::MissingPropertyDescription,
                ));
            };

            match parsed {
                Ok(HomieValue::Enum(value)) => match value.as_str().try_into() {
                    Ok(mode) => ParseOutcome::Parsed(ThermostatNodeSetEvents::Mode(mode)),
                    Err(_) => ParseOutcome::Invalid(ParseError::new(
                        property.prop_id().to_string(),
                        set_value,
                        ParseErrorKind::InvalidVariant,
                    )),
                },
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &THERMOSTAT_NODE_BOOST_STATE_PROP.id) {
            let Some(parsed) = desc.with_property(property, |prop_desc| {
                HomieValue::parse(set_value, prop_desc)
            }) else {
                return ParseOutcome::Invalid(ParseError::new(
                    property_id,
                    set_value,
                    ParseErrorKind::MissingPropertyDescription,
                ));
            };

            match parsed {
                Ok(HomieValue::Bool(value)) => {
                    ParseOutcome::Parsed(ThermostatNodeSetEvents::Boost(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else {
            ParseOutcome::NoMatch
        }
    }

    fn parse_set_event(
        &self,
        desc: &HomieDeviceDescription,
        event: &Homie5Message,
    ) -> ParseOutcome<Self::Event> {
        match event {
            Homie5Message::PropertySet {
                property,
                set_value,
            } => self.parse_set(property, desc, set_value),
            _ => ParseOutcome::Invalid(ParseError::new(
                THERMOSTAT_NODE_SET_TEMPERATURE_PROP.id.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

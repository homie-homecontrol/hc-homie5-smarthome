use homie5::{
    HOMIE_UNIT_DEGREE_CELSIUS, HOMIE_UNIT_PERCENT, HOMIE_UNIT_SECONDS, Homie5DeviceProtocol,
    Homie5Message, Homie5ProtocolError, HomieID, HomieValue, NodeRef, PropertyRef,
    device_description::{
        BooleanFormat, FloatRange, HomieDeviceDescription, HomieNodeDescription,
        HomiePropertyFormat, IntegerRange, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_TYPE_THERMOSTAT, SetCommandParser,
};

pub const THERMOSTAT_NODE_DEFAULT_ID: HomieID = HomieID::new_const("thermostat");
pub const THERMOSTAT_NODE_DEFAULT_NAME: &str = "Thermostat";
pub const THERMOSTAT_NODE_SET_TEMPERATURE_PROP_ID: HomieID = HomieID::new_const("set-temperature");
pub const THERMOSTAT_NODE_VALVE_PROP_ID: HomieID = HomieID::new_const("valve");
pub const THERMOSTAT_NODE_MODE_PROP_ID: HomieID = HomieID::new_const("mode");
pub const THERMOSTAT_NODE_WINDOWOPEN_PROP_ID: HomieID = HomieID::new_const("window-open");
pub const THERMOSTAT_NODE_BOOST_STATE_PROP_ID: HomieID = HomieID::new_const("boost-state");
pub const THERMOSTAT_NODE_BOOST_TIME_PROP_ID: HomieID = HomieID::new_const("boost-time");

#[derive(Debug)]
pub struct ThermostatNode {
    pub publisher: ThermostatNodePublisher,
    pub set_temperature: f64,
    pub set_temperature_target: f64,
    pub valve: Option<i64>,
    pub mode: Option<ThermostatNodeModes>,
    pub windowopen: Option<bool>,
    pub boost_state: Option<bool>,
    pub boost_time: Option<i64>,
}

#[derive(Debug, Default, Copy, PartialEq, Clone, Serialize, Deserialize)]
pub enum ThermostatNodeModes {
    #[default]
    Off,
    Auto,
    Manual,
    Party,
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
    pub boost_time: bool,
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
            boost_time: true,
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
        .r#type(SMARTHOME_TYPE_THERMOSTAT);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &ThermostatNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            THERMOSTAT_NODE_SET_TEMPERATURE_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                .name("Set target temperature")
                .format(HomiePropertyFormat::FloatRange(config.temp_range.clone()))
                .unit(config.unit.to_owned())
                .settable(true)
                .retained(true)
                .build(),
        )
        .add_property_cond(THERMOSTAT_NODE_VALVE_PROP_ID, config.valve, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Valve opening Level")
                .format(HomiePropertyFormat::IntegerRange(IntegerRange {
                    min: Some(0),
                    max: Some(100),
                    step: None,
                }))
                .unit(HOMIE_UNIT_PERCENT)
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(
            THERMOSTAT_NODE_WINDOWOPEN_PROP_ID,
            config.windowopen,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                    .name("Window open detected")
                    .format(HomiePropertyFormat::Boolean(BooleanFormat {
                        false_val: "closed".to_string(),
                        true_val: "open".to_string(),
                    }))
                    .settable(false)
                    .retained(true)
                    .build()
            },
        )
        .add_property_cond(
            THERMOSTAT_NODE_BOOST_STATE_PROP_ID,
            config.boost_state,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                    .name("Boost mode active")
                    .settable(true)
                    .retained(true)
                    .build()
            },
        )
        .add_property_cond(
            THERMOSTAT_NODE_BOOST_TIME_PROP_ID,
            config.boost_time,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                    .name("Seconds remaining for boost")
                    .format(HomiePropertyFormat::IntegerRange(IntegerRange {
                        min: Some(0),
                        max: None,
                        step: None,
                    }))
                    .unit(HOMIE_UNIT_SECONDS)
                    .settable(false)
                    .retained(false)
                    .build()
            },
        )
        .add_property_cond(THERMOSTAT_NODE_MODE_PROP_ID, config.mode, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Mode")
                .format(HomiePropertyFormat::Enum(
                    config.modes.iter().map(|m| m.into()).collect(),
                ))
                .settable(true)
                .retained(true)
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
    use homie5::device_description::HomiePropertyFormat;

    fn base_disabled_config() -> ThermostatNodeConfig {
        ThermostatNodeConfig {
            unit: "F".to_string(),
            valve: false,
            windowopen: false,
            boost_state: false,
            boost_time: false,
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
            .get(&THERMOSTAT_NODE_SET_TEMPERATURE_PROP_ID)
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
            THERMOSTAT_NODE_VALVE_PROP_ID,
            THERMOSTAT_NODE_WINDOWOPEN_PROP_ID,
            THERMOSTAT_NODE_BOOST_STATE_PROP_ID,
            THERMOSTAT_NODE_BOOST_TIME_PROP_ID,
            THERMOSTAT_NODE_MODE_PROP_ID,
        ];

        for expected_property_id in gated_properties.iter() {
            let mut config = base_disabled_config();
            if expected_property_id == &THERMOSTAT_NODE_VALVE_PROP_ID {
                config.valve = true;
            } else if expected_property_id == &THERMOSTAT_NODE_WINDOWOPEN_PROP_ID {
                config.windowopen = true;
            } else if expected_property_id == &THERMOSTAT_NODE_BOOST_STATE_PROP_ID {
                config.boost_state = true;
            } else if expected_property_id == &THERMOSTAT_NODE_BOOST_TIME_PROP_ID {
                config.boost_time = true;
            } else if expected_property_id == &THERMOSTAT_NODE_MODE_PROP_ID {
                config.mode = true;
            } else {
                unreachable!("unknown thermostat property id");
            }

            let node = ThermostatNodeBuilder::new(&config).build();

            assert!(
                node.properties
                    .contains_key(&THERMOSTAT_NODE_SET_TEMPERATURE_PROP_ID),
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
            .get(&THERMOSTAT_NODE_MODE_PROP_ID)
            .expect("mode property must exist when mode config is enabled");

        assert_eq!(
            mode_property.format,
            HomiePropertyFormat::Enum(vec!["cool".to_string(), "heat".to_string()])
        );
    }
}

#[derive(Debug)]
pub struct ThermostatNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    set_temperature_prop: HomieID,
    boost_prop: HomieID,
    boost_time_prop: HomieID,
    mode_prop: HomieID,
    valve_prop: HomieID,
    windowopen_prop: HomieID,
}

impl ThermostatNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            mode_prop: THERMOSTAT_NODE_MODE_PROP_ID,
            boost_prop: THERMOSTAT_NODE_BOOST_STATE_PROP_ID,
            boost_time_prop: THERMOSTAT_NODE_BOOST_TIME_PROP_ID,
            valve_prop: THERMOSTAT_NODE_VALVE_PROP_ID,
            windowopen_prop: THERMOSTAT_NODE_WINDOWOPEN_PROP_ID,
            set_temperature_prop: THERMOSTAT_NODE_SET_TEMPERATURE_PROP_ID,
        }
    }

    pub fn set_temperature(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.set_temperature_prop,
            value.to_string(),
            true,
        )
    }

    pub fn set_temperature_target(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &self.set_temperature_prop,
            value.to_string(),
            true,
        )
    }

    pub fn mode(&self, mode: ThermostatNodeModes) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.mode_prop, &mode, true)
    }

    pub fn mode_target(&self, mode: ThermostatNodeModes) -> homie5::client::Publish {
        self.client
            .publish_target(self.node.node_id(), &self.mode_prop, &mode, true)
    }

    pub fn boost(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.boost_prop,
            value.to_string(),
            true,
        )
    }

    pub fn boost_time(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.boost_time_prop,
            value.to_string(),
            true,
        )
    }

    pub fn valve(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.valve_prop,
            value.to_string(),
            true,
        )
    }

    pub fn windowopen(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.windowopen_prop,
            value.to_string(),
            true,
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

        if property.match_with_node(&self.node, &self.set_temperature_prop) {
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
        } else if property.match_with_node(&self.node, &self.mode_prop) {
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
        } else if property.match_with_node(&self.node, &self.boost_prop) {
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
                self.set_temperature_prop.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

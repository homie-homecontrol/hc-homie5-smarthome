use homie5::{
    Homie5DeviceProtocol, Homie5Message, HomieID, HomieValue, NodeRef, PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat, IntegerRange,
        NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_CAP_ALARM, SetCommandParser};

pub const ALARM_NODE_DEFAULT_ID: HomieID = HomieID::new_const("alarm");
pub const ALARM_NODE_DEFAULT_NAME: &str = "Alarm";
pub const ALARM_NODE_STATE_PROP_ID: HomieID = HomieID::new_const("state");
pub const ALARM_NODE_SOUND_PROP_ID: HomieID = HomieID::new_const("sound");
pub const ALARM_NODE_DURATION_PROP_ID: HomieID = HomieID::new_const("duration");

// ── Node (state) ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct AlarmNode {
    pub publisher: AlarmNodePublisher,
    pub state: bool,
    pub sound: Option<String>,
    pub duration: Option<i64>,
}

// ── Set events ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum AlarmNodeSetEvents {
    State(bool),
    Sound(String),
    Duration(i64),
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AlarmNodeConfig {
    pub sound: bool,
    pub sounds: Vec<String>,
    pub duration: bool,
}

impl Default for AlarmNodeConfig {
    fn default() -> Self {
        Self {
            sound: false,
            sounds: vec!["default".to_owned()],
            duration: false,
        }
    }
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct AlarmNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl AlarmNodeBuilder {
    pub fn new(config: &AlarmNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(ALARM_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_ALARM);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &AlarmNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            ALARM_NODE_STATE_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                .name("Alarm state")
                .settable(true)
                .retained(true)
                .build(),
        )
        .add_property_cond(ALARM_NODE_SOUND_PROP_ID, config.sound, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Alarm sound")
                .format(HomiePropertyFormat::Enum(config.sounds.clone()))
                .settable(true)
                .retained(true)
                .build()
        })
        .add_property_cond(ALARM_NODE_DURATION_PROP_ID, config.duration, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Alarm duration")
                .unit("s")
                .format(HomiePropertyFormat::IntegerRange(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                }))
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
    ) -> (HomieNodeDescription, AlarmNodePublisher) {
        (
            self.node_builder.build(),
            AlarmNodePublisher::new(
                NodeRef::new(
                    client.homie_domain().to_owned(),
                    client.id().clone(),
                    node_id,
                ),
                client.clone(),
            ),
        )
    }
}

// ── Publisher ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct AlarmNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    state_prop: HomieID,
    sound_prop: HomieID,
    duration_prop: HomieID,
}

impl AlarmNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            state_prop: ALARM_NODE_STATE_PROP_ID,
            sound_prop: ALARM_NODE_SOUND_PROP_ID,
            duration_prop: ALARM_NODE_DURATION_PROP_ID,
        }
    }

    pub fn state(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.state_prop,
            value.to_string(),
            true,
        )
    }

    pub fn sound(&self, value: &str) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.sound_prop, value, true)
    }

    pub fn duration(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.duration_prop,
            value.to_string(),
            true,
        )
    }
}

impl SetCommandParser for AlarmNodePublisher {
    type Event = AlarmNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &self.state_prop) {
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
                    ParseOutcome::Parsed(AlarmNodeSetEvents::State(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &self.sound_prop) {
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
                Ok(HomieValue::Enum(value)) => {
                    ParseOutcome::Parsed(AlarmNodeSetEvents::Sound(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &self.duration_prop) {
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
                Ok(HomieValue::Integer(value)) => {
                    ParseOutcome::Parsed(AlarmNodeSetEvents::Duration(value))
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
                self.state_prop.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

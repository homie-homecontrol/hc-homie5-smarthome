use std::{fmt::Display, str::FromStr};

use homie5::{
    HOMIE_UNIT_PERCENT, Homie5DeviceProtocol, Homie5Message, Homie5ProtocolError, HomieID,
    HomieValue, NodeRef, PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, IntegerRange, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, PropSpec, SMARTHOME_CAP_SHUTTER, SetCommandParser,
};

pub const SHUTTER_NODE_DEFAULT_ID: HomieID = HomieID::new_const("shutter");
pub const SHUTTER_NODE_DEFAULT_NAME: &str = "Shutter control";

pub const SHUTTER_NODE_POSITION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("position"),
    retained: true,
    settable: true,
};
pub const SHUTTER_NODE_ACTION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("action"),
    retained: false,
    settable: true,
};

#[derive(Debug)]
pub struct ShutterNode {
    pub publisher: ShutterNodePublisher,
    pub position: i64,
    pub position_target: i64,
}

#[derive(Debug)]
pub enum ShutterNodeActions {
    Up,
    Down,
    Stop,
}

impl Display for ShutterNodeActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &'static str = self.into();
        write!(f, "{}", s)
    }
}

impl From<&ShutterNodeActions> for &'static str {
    fn from(action: &ShutterNodeActions) -> Self {
        match action {
            ShutterNodeActions::Up => "up",
            ShutterNodeActions::Down => "down",
            ShutterNodeActions::Stop => "stop",
        }
    }
}
impl FromStr for ShutterNodeActions {
    type Err = Homie5ProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "up" => Ok(ShutterNodeActions::Up),
            "down" => Ok(ShutterNodeActions::Down),
            "stop" => Ok(ShutterNodeActions::Stop),
            _ => Err(Homie5ProtocolError::InvalidPayload),
        }
    }
}

#[derive(Debug)]
pub enum ShutterNodeSetEvents {
    Position(i64),
    Action(ShutterNodeActions),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShutterNodeConfig {
    pub can_stop: bool,
}

impl Default for ShutterNodeConfig {
    fn default() -> Self {
        Self { can_stop: true }
    }
}

pub struct ShutterNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl ShutterNodeBuilder {
    pub fn new(config: &ShutterNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(SHUTTER_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_SHUTTER);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &ShutterNodeConfig,
    ) -> NodeDescriptionBuilder {
        let mut actions = vec![ShutterNodeActions::Up, ShutterNodeActions::Down];

        if config.can_stop {
            actions.push(ShutterNodeActions::Stop);
        }

        db.add_property(
            SHUTTER_NODE_POSITION_PROP.id,
            PropertyDescriptionBuilder::integer()
                .name("Shutter position")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: Some(100),
                    step: None,
                })
                .unit(HOMIE_UNIT_PERCENT)
                .settable(SHUTTER_NODE_POSITION_PROP.settable)
                .retained(SHUTTER_NODE_POSITION_PROP.retained)
                .build(),
        )
        .add_property(
            SHUTTER_NODE_ACTION_PROP.id,
            PropertyDescriptionBuilder::enumeration(actions.iter().map(|a| a.to_string()))
                .unwrap()
                .name("Control Shutter")
                .settable(SHUTTER_NODE_ACTION_PROP.settable)
                .retained(SHUTTER_NODE_ACTION_PROP.retained)
                .build(),
        )
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
    ) -> (HomieNodeDescription, ShutterNodePublisher) {
        let did = client.id().clone();
        (
            self.node_builder.build(),
            ShutterNodePublisher::new(
                NodeRef::new(client.homie_domain().to_owned(), did, node_id),
                client.clone(),
            ),
        )
    }
}

#[derive(Debug)]
pub struct ShutterNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
}

impl ShutterNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn position(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &SHUTTER_NODE_POSITION_PROP.id,
            value.to_string(),
            SHUTTER_NODE_POSITION_PROP.retained,
        )
    }

    pub fn position_target(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &SHUTTER_NODE_POSITION_PROP.id,
            value.to_string(),
            SHUTTER_NODE_POSITION_PROP.retained,
        )
    }

    pub fn action(&self, action: ShutterNodeActions) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &SHUTTER_NODE_ACTION_PROP.id,
            action.to_string(),
            SHUTTER_NODE_ACTION_PROP.retained,
        )
    }
}

impl SetCommandParser for ShutterNodePublisher {
    type Event = ShutterNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &SHUTTER_NODE_POSITION_PROP.id) {
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
                    ParseOutcome::Parsed(ShutterNodeSetEvents::Position(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &SHUTTER_NODE_ACTION_PROP.id) {
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
                Ok(HomieValue::Enum(value)) => match ShutterNodeActions::from_str(&value) {
                    Ok(action) => ParseOutcome::Parsed(ShutterNodeSetEvents::Action(action)),
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
                SHUTTER_NODE_POSITION_PROP.id.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

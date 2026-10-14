use std::str::FromStr;

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
    ParseError, ParseErrorKind, ParseOutcome, PropSpec, SMARTHOME_CAP_LEVEL, SetCommandParser,
};

pub const LEVEL_NODE_DEFAULT_ID: HomieID = HomieID::new_const("level");
pub const LEVEL_NODE_DEFAULT_NAME: &str = "Level control";

pub const LEVEL_NODE_VALUE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("value"),
    retained: true,
    settable: true,
};
pub const LEVEL_NODE_ACTION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("action"),
    retained: false,
    settable: true,
};

#[derive(Debug)]
pub struct LevelNode {
    pub publisher: LevelNodePublisher,
    pub value: i64,
    pub value_target: i64,
}

#[derive(Debug)]
pub enum LevelNodeActions {
    StepUp,
    StepDown,
}

impl FromStr for LevelNodeActions {
    type Err = Homie5ProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "step-up" => Ok(LevelNodeActions::StepUp),
            "step-down" => Ok(LevelNodeActions::StepDown),
            _ => Err(Homie5ProtocolError::InvalidPayload),
        }
    }
}

#[derive(Debug)]
pub enum LevelNodeSetEvents {
    Value(i64),
    Action(LevelNodeActions),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LevelNodeConfig {
    pub settable: bool,
    pub step_action: bool,
}

impl Default for LevelNodeConfig {
    fn default() -> Self {
        Self {
            settable: true,
            step_action: true,
        }
    }
}

pub struct LevelNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl LevelNodeBuilder {
    pub fn new(config: &LevelNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(LEVEL_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_LEVEL);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &LevelNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            LEVEL_NODE_VALUE_PROP.id,
            PropertyDescriptionBuilder::integer()
                .name("Level")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: Some(100),
                    step: None,
                })
                .unit(HOMIE_UNIT_PERCENT)
                .settable(LEVEL_NODE_VALUE_PROP.settable && config.settable)
                .retained(LEVEL_NODE_VALUE_PROP.retained)
                .build(),
        )
        .add_property_cond(LEVEL_NODE_ACTION_PROP.id, config.step_action, || {
            PropertyDescriptionBuilder::enumeration(["step-up", "step-down"])
                .unwrap()
                .name("Step level")
                .settable(LEVEL_NODE_ACTION_PROP.settable && config.settable)
                .retained(LEVEL_NODE_ACTION_PROP.retained)
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
    ) -> (HomieNodeDescription, LevelNodePublisher) {
        let did = client.id().clone();
        (
            self.node_builder.build(),
            LevelNodePublisher::new(
                NodeRef::new(client.homie_domain().to_owned(), did, node_id),
                client.clone(),
            ),
        )
    }
}

#[derive(Debug)]
pub struct LevelNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
}

impl LevelNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn value(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &LEVEL_NODE_VALUE_PROP.id,
            value.to_string(),
            LEVEL_NODE_VALUE_PROP.retained,
        )
    }

    pub fn value_target(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &LEVEL_NODE_VALUE_PROP.id,
            value.to_string(),
            LEVEL_NODE_VALUE_PROP.retained,
        )
    }

    pub fn action(&self, action: LevelNodeActions) -> homie5::client::Publish {
        let action_str = match action {
            LevelNodeActions::StepUp => "step-up",
            LevelNodeActions::StepDown => "step-down",
        };
        self.client.publish_value(
            self.node.node_id(),
            &LEVEL_NODE_ACTION_PROP.id,
            action_str,
            LEVEL_NODE_ACTION_PROP.retained,
        )
    }
}

impl SetCommandParser for LevelNodePublisher {
    type Event = LevelNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &LEVEL_NODE_VALUE_PROP.id) {
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
                    ParseOutcome::Parsed(LevelNodeSetEvents::Value(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &LEVEL_NODE_ACTION_PROP.id) {
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
                Ok(HomieValue::Enum(value)) => match LevelNodeActions::from_str(&value) {
                    Ok(action) => ParseOutcome::Parsed(LevelNodeSetEvents::Action(action)),
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
                LEVEL_NODE_VALUE_PROP.id.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

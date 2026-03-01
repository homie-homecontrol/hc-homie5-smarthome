use std::str::FromStr;

use homie5::{
    HOMIE_UNIT_PERCENT, Homie5DeviceProtocol, Homie5Message, Homie5ProtocolError, HomieID,
    HomieValue, NodeRef, PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat, IntegerRange,
        NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_CAP_LEVEL, SetCommandParser};

pub const LEVEL_NODE_DEFAULT_ID: HomieID = HomieID::new_const("level");
pub const LEVEL_NODE_DEFAULT_NAME: &str = "Level control";
pub const LEVEL_NODE_VALUE_PROP_ID: HomieID = HomieID::new_const("value");
pub const LEVEL_NODE_ACTION_PROP_ID: HomieID = HomieID::new_const("action");

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
            LEVEL_NODE_VALUE_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Level")
                .format(HomiePropertyFormat::IntegerRange(IntegerRange {
                    min: Some(0),
                    max: Some(100),
                    step: None,
                }))
                .unit(HOMIE_UNIT_PERCENT)
                .settable(config.settable)
                .retained(true)
                .build(),
        )
        .add_property_cond(LEVEL_NODE_ACTION_PROP_ID, config.step_action, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Step level")
                .format(HomiePropertyFormat::Enum(vec![
                    "step-up".to_owned(),
                    "step-down".to_owned(),
                ]))
                .settable(config.settable)
                .retained(false)
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
    value_prop: HomieID,
    action_prop: HomieID,
}

impl LevelNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            value_prop: LEVEL_NODE_VALUE_PROP_ID,
            action_prop: LEVEL_NODE_ACTION_PROP_ID,
        }
    }

    pub fn value(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.value_prop,
            value.to_string(),
            true,
        )
    }

    pub fn value_target(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &self.value_prop,
            value.to_string(),
            true,
        )
    }

    pub fn action(&self, action: LevelNodeActions) -> homie5::client::Publish {
        let action_str = match action {
            LevelNodeActions::StepUp => "step-up",
            LevelNodeActions::StepDown => "step-down",
        };
        self.client
            .publish_value(self.node.node_id(), &self.action_prop, action_str, false)
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

        if property.match_with_node(&self.node, &self.value_prop) {
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
        } else if property.match_with_node(&self.node, &self.action_prop) {
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
                self.value_prop.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

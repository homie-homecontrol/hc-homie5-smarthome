use core::fmt;
use std::str::FromStr;

use homie5::{
    Homie5DeviceProtocol, Homie5Message, Homie5ProtocolError, HomieID, HomieValue, NodeRef,
    PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, PropSpec, SMARTHOME_CAP_LOCK, SetCommandParser,
};

pub const LOCK_NODE_DEFAULT_ID: HomieID = HomieID::new_const("lock");
pub const LOCK_NODE_DEFAULT_NAME: &str = "Lock control";

pub const LOCK_NODE_STATE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("state"),
    retained: true,
    settable: true,
};
pub const LOCK_NODE_ACTION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("action"),
    retained: false,
    settable: true,
};

#[derive(Debug)]
pub struct LockNode {
    pub publisher: LockNodePublisher,
    pub state: bool,
    pub state_target: bool,
}

#[derive(Debug)]
pub enum LockNodeActions {
    Lock,
    Unlock,
    Toggle,
}

impl fmt::Display for LockNodeActions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl LockNodeActions {
    pub fn as_str(&self) -> &'static str {
        match self {
            LockNodeActions::Lock => "lock",
            LockNodeActions::Unlock => "unlock",
            LockNodeActions::Toggle => "toggle",
        }
    }
}

impl FromStr for LockNodeActions {
    type Err = Homie5ProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lock" => Ok(LockNodeActions::Lock),
            "unlock" => Ok(LockNodeActions::Unlock),
            "toggle" => Ok(LockNodeActions::Toggle),
            _ => Err(Homie5ProtocolError::InvalidPayload),
        }
    }
}

#[derive(Debug)]
pub enum LockNodeSetEvents {
    State(bool),
    Action(LockNodeActions),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LockNodeConfig {
    pub settable: bool,
}

impl Default for LockNodeConfig {
    fn default() -> Self {
        Self { settable: true }
    }
}

pub struct LockNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl LockNodeBuilder {
    pub fn new(config: &LockNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(LOCK_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_LOCK);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &LockNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            LOCK_NODE_STATE_PROP.id,
            PropertyDescriptionBuilder::boolean()
                .name("Lock state")
                .boolean_labels("unlocked", "locked")
                .settable(LOCK_NODE_STATE_PROP.settable && config.settable)
                .retained(LOCK_NODE_STATE_PROP.retained)
                .build(),
        )
        .add_property(
            LOCK_NODE_ACTION_PROP.id,
            PropertyDescriptionBuilder::enumeration(["lock", "unlock", "toggle"])
                .unwrap()
                .name("Lock action")
                .settable(LOCK_NODE_ACTION_PROP.settable && config.settable)
                .retained(LOCK_NODE_ACTION_PROP.retained)
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
    ) -> (HomieNodeDescription, LockNodePublisher) {
        (
            self.node_builder.build(),
            LockNodePublisher::new(
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

#[derive(Debug)]
pub struct LockNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
}

impl LockNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn state(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &LOCK_NODE_STATE_PROP.id,
            value.to_string(),
            LOCK_NODE_STATE_PROP.retained,
        )
    }

    pub fn state_target(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &LOCK_NODE_STATE_PROP.id,
            value.to_string(),
            LOCK_NODE_STATE_PROP.retained,
        )
    }

    pub fn action(&self, action: &LockNodeActions) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &LOCK_NODE_ACTION_PROP.id,
            action.as_str(),
            LOCK_NODE_ACTION_PROP.retained,
        )
    }
}

impl SetCommandParser for LockNodePublisher {
    type Event = LockNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &LOCK_NODE_STATE_PROP.id) {
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
                    ParseOutcome::Parsed(LockNodeSetEvents::State(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &LOCK_NODE_ACTION_PROP.id) {
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
                Ok(HomieValue::Enum(value)) => match LockNodeActions::from_str(&value) {
                    Ok(action) => ParseOutcome::Parsed(LockNodeSetEvents::Action(action)),
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
                LOCK_NODE_STATE_PROP.id.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

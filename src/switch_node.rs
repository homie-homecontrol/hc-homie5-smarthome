use core::fmt;

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
    ParseError, ParseErrorKind, ParseOutcome, PropSpec, SMARTHOME_CAP_SWITCH, SetCommandParser,
};

pub const SWITCH_NODE_DEFAULT_ID: HomieID = HomieID::new_const("switch");
pub const SWITCH_NODE_DEFAULT_NAME: &str = "On/Off switch";

pub const SWITCH_NODE_STATE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("state"),
    retained: true,
    settable: true,
};
pub const SWITCH_NODE_ACTION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("action"),
    retained: false,
    settable: true,
};

#[derive(Debug)]
pub struct SwitchNode {
    pub publisher: SwitchNodePublisher,
    pub state: bool,
    pub state_target: bool,
}

#[derive(Debug)]
pub enum SwitchNodeActions {
    Toggle,
}

impl fmt::Display for SwitchNodeActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwitchNodeActions::Toggle => f.write_str("toggle"),
        }
    }
}

impl TryFrom<String> for SwitchNodeActions {
    type Error = Homie5ProtocolError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "toggle" => Ok(SwitchNodeActions::Toggle),
            _ => Err(Homie5ProtocolError::InvalidPayload),
        }
    }
}

#[derive(Debug)]
pub enum SwitchNodeSetEvents {
    State(bool),
    Action(SwitchNodeActions),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SwitchNodeConfig {
    pub settable: bool,
}

impl Default for SwitchNodeConfig {
    fn default() -> Self {
        Self { settable: true }
    }
}

pub struct SwitchNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl SwitchNodeBuilder {
    pub fn new(config: &SwitchNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(SWITCH_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_SWITCH);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &SwitchNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            SWITCH_NODE_STATE_PROP.id,
            PropertyDescriptionBuilder::boolean()
                .name("On/Off state")
                .boolean_labels("off", "on")
                .settable(SWITCH_NODE_STATE_PROP.settable && config.settable)
                .retained(SWITCH_NODE_STATE_PROP.retained)
                .build(),
        )
        .add_property(
            SWITCH_NODE_ACTION_PROP.id,
            PropertyDescriptionBuilder::enumeration(["toggle"])
                .unwrap()
                .name("Change state")
                .settable(SWITCH_NODE_ACTION_PROP.settable && config.settable)
                .retained(SWITCH_NODE_ACTION_PROP.retained)
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
    ) -> (HomieNodeDescription, SwitchNodePublisher) {
        (
            self.node_builder.build(),
            SwitchNodePublisher::new(
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
pub struct SwitchNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
}

impl SwitchNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn state(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &SWITCH_NODE_STATE_PROP.id,
            value.to_string(),
            SWITCH_NODE_STATE_PROP.retained,
        )
    }

    pub fn state_target(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &SWITCH_NODE_STATE_PROP.id,
            value.to_string(),
            SWITCH_NODE_STATE_PROP.retained,
        )
    }

    pub fn action(&self) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &SWITCH_NODE_ACTION_PROP.id,
            SwitchNodeActions::Toggle.to_string(),
            SWITCH_NODE_ACTION_PROP.retained,
        )
    }
}

impl SetCommandParser for SwitchNodePublisher {
    type Event = SwitchNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_device(
            self.client.device_ref(),
            self.node.node_id(),
            &SWITCH_NODE_STATE_PROP.id,
        ) {
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
                    ParseOutcome::Parsed(SwitchNodeSetEvents::State(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &SWITCH_NODE_ACTION_PROP.id) {
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
                Ok(HomieValue::Enum(value)) => match value.try_into() {
                    Ok(value) => ParseOutcome::Parsed(SwitchNodeSetEvents::Action(value)),
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
                SWITCH_NODE_STATE_PROP.id.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

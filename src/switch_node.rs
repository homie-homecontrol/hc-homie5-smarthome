use core::fmt;

use homie5::{
    device_description::{
        BooleanFormat, HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat,
        NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
    Homie5DeviceProtocol, Homie5Message, Homie5ProtocolError, HomieID, HomieValue, NodeRef,
    PropertyRef,
};
use serde::{Deserialize, Serialize};

use crate::{ParseError, ParseErrorKind, ParseOutcome, SetCommandParser, SMARTHOME_TYPE_SWITCH};

pub const SWITCH_NODE_DEFAULT_ID: HomieID = HomieID::new_const("switch");
pub const SWITCH_NODE_DEFAULT_NAME: &str = "On/Off switch";
pub const SWITCH_NODE_STATE_PROP_ID: HomieID = HomieID::new_const("state");
pub const SWITCH_NODE_ACTION_PROP_ID: HomieID = HomieID::new_const("action");

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
        .r#type(SMARTHOME_TYPE_SWITCH);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &SwitchNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            SWITCH_NODE_STATE_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                .name("On/Off state")
                .format(HomiePropertyFormat::Boolean(BooleanFormat {
                    false_val: "off".to_owned(),
                    true_val: "on".to_owned(),
                }))
                .settable(config.settable)
                .retained(true)
                .build(),
        )
        .add_property(
            SWITCH_NODE_ACTION_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Change state")
                .format(HomiePropertyFormat::Enum(vec!["toggle".to_owned()]))
                .settable(config.settable)
                .retained(false)
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
    state_prop: HomieID,
    action_prop: HomieID,
}

impl SwitchNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            state_prop: SWITCH_NODE_STATE_PROP_ID,
            action_prop: SWITCH_NODE_ACTION_PROP_ID,
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

    pub fn state_target(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &self.state_prop,
            value.to_string(),
            true,
        )
    }

    pub fn action(&self) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.action_prop,
            SwitchNodeActions::Toggle.to_string(),
            false,
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
            &self.state_prop,
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
                self.state_prop.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

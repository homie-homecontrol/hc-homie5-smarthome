use core::fmt;
use std::str::FromStr;

use homie5::{
    Homie5DeviceProtocol, Homie5Message, Homie5ProtocolError, HomieID, HomieValue, NodeRef,
    PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_CAP_GARAGE_DOOR, SetCommandParser,
};

pub const GARAGE_DOOR_NODE_DEFAULT_ID: HomieID = HomieID::new_const("garage-door");
pub const GARAGE_DOOR_NODE_DEFAULT_NAME: &str = "Garage door";
pub const GARAGE_DOOR_NODE_STATE_PROP_ID: HomieID = HomieID::new_const("state");
pub const GARAGE_DOOR_NODE_ACTION_PROP_ID: HomieID = HomieID::new_const("action");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GarageDoorState {
    Open,
    Closed,
    Opening,
    Closing,
    Stopped,
    Unknown,
}

impl GarageDoorState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Opening => "opening",
            Self::Closing => "closing",
            Self::Stopped => "stopped",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for GarageDoorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GarageDoorAction {
    Open,
    Close,
    Trigger,
    Stop,
}

impl GarageDoorAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Close => "close",
            Self::Trigger => "trigger",
            Self::Stop => "stop",
        }
    }
}

impl fmt::Display for GarageDoorAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for GarageDoorAction {
    type Err = Homie5ProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "close" => Ok(Self::Close),
            "trigger" => Ok(Self::Trigger),
            "stop" => Ok(Self::Stop),
            _ => Err(Homie5ProtocolError::InvalidPayload),
        }
    }
}

#[derive(Debug)]
pub struct GarageDoorNode {
    pub publisher: GarageDoorNodePublisher,
    pub state: GarageDoorState,
}

#[derive(Debug)]
pub enum GarageDoorNodeSetEvents {
    Action(GarageDoorAction),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GarageDoorNodeConfig {
    pub settable: bool,
    pub action_open: bool,
    pub action_close: bool,
    pub action_trigger: bool,
    pub action_stop: bool,
    pub state_opening: bool,
    pub state_closing: bool,
    pub state_stopped: bool,
    pub state_unknown: bool,
}

impl Default for GarageDoorNodeConfig {
    fn default() -> Self {
        Self {
            settable: true,
            action_open: true,
            action_close: true,
            action_trigger: true,
            action_stop: true,
            state_opening: true,
            state_closing: true,
            state_stopped: true,
            state_unknown: true,
        }
    }
}

impl GarageDoorNodeConfig {
    fn build_action_format(&self) -> Vec<String> {
        let mut actions = Vec::new();
        if self.action_open {
            actions.push(GarageDoorAction::Open.as_str().to_owned());
        }
        if self.action_close {
            actions.push(GarageDoorAction::Close.as_str().to_owned());
        }
        if self.action_trigger {
            actions.push(GarageDoorAction::Trigger.as_str().to_owned());
        }
        if self.action_stop {
            actions.push(GarageDoorAction::Stop.as_str().to_owned());
        }

        if actions.is_empty() {
            actions.push(GarageDoorAction::Trigger.as_str().to_owned());
        }

        actions
    }

    fn build_state_format(&self) -> Vec<String> {
        let mut states = vec![
            GarageDoorState::Open.as_str().to_owned(),
            GarageDoorState::Closed.as_str().to_owned(),
        ];

        if self.state_opening {
            states.push(GarageDoorState::Opening.as_str().to_owned());
        }
        if self.state_closing {
            states.push(GarageDoorState::Closing.as_str().to_owned());
        }
        if self.state_stopped {
            states.push(GarageDoorState::Stopped.as_str().to_owned());
        }
        if self.state_unknown {
            states.push(GarageDoorState::Unknown.as_str().to_owned());
        }

        states
    }
}

pub struct GarageDoorNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl GarageDoorNodeBuilder {
    pub fn new(config: &GarageDoorNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(GARAGE_DOOR_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_GARAGE_DOOR);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &GarageDoorNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            GARAGE_DOOR_NODE_STATE_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Garage door state")
                .format(HomiePropertyFormat::Enum(config.build_state_format()))
                .settable(false)
                .retained(true)
                .build(),
        )
        .add_property(
            GARAGE_DOOR_NODE_ACTION_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Garage door action")
                .format(HomiePropertyFormat::Enum(config.build_action_format()))
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
    ) -> (HomieNodeDescription, GarageDoorNodePublisher) {
        (
            self.node_builder.build(),
            GarageDoorNodePublisher::new(
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
pub struct GarageDoorNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    state_prop: HomieID,
    action_prop: HomieID,
}

impl GarageDoorNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            state_prop: GARAGE_DOOR_NODE_STATE_PROP_ID,
            action_prop: GARAGE_DOOR_NODE_ACTION_PROP_ID,
        }
    }

    pub fn state(&self, state: GarageDoorState) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.state_prop, state.as_str(), true)
    }

    pub fn action(&self, action: GarageDoorAction) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.action_prop,
            action.as_str(),
            false,
        )
    }
}

impl SetCommandParser for GarageDoorNodePublisher {
    type Event = GarageDoorNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &self.action_prop) {
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
                Ok(HomieValue::Enum(value)) => match GarageDoorAction::from_str(&value) {
                    Ok(action) => ParseOutcome::Parsed(GarageDoorNodeSetEvents::Action(action)),
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
                self.action_prop.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

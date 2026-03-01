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
    ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_CAP_MEDIAPLAYER, SetCommandParser,
};

pub const MEDIAPLAYER_NODE_DEFAULT_ID: HomieID = HomieID::new_const("mediaplayer");
pub const MEDIAPLAYER_NODE_DEFAULT_NAME: &str = "Media player";
pub const MEDIAPLAYER_NODE_ACTION_PROP_ID: HomieID = HomieID::new_const("action");
pub const MEDIAPLAYER_NODE_STATE_PROP_ID: HomieID = HomieID::new_const("state");
pub const MEDIAPLAYER_NODE_SHUFFLE_PROP_ID: HomieID = HomieID::new_const("shuffle");
pub const MEDIAPLAYER_NODE_REPEAT_PROP_ID: HomieID = HomieID::new_const("repeat");

// ── Actions ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaplayerAction {
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    Forward,
    Rewind,
}

impl MediaplayerAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Play => "play",
            Self::Pause => "pause",
            Self::Stop => "stop",
            Self::Next => "next",
            Self::Previous => "previous",
            Self::Forward => "forward",
            Self::Rewind => "rewind",
        }
    }
}

impl fmt::Display for MediaplayerAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for MediaplayerAction {
    type Err = Homie5ProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "play" => Ok(Self::Play),
            "pause" => Ok(Self::Pause),
            "stop" => Ok(Self::Stop),
            "next" => Ok(Self::Next),
            "previous" => Ok(Self::Previous),
            "forward" => Ok(Self::Forward),
            "rewind" => Ok(Self::Rewind),
            _ => Err(Homie5ProtocolError::InvalidPayload),
        }
    }
}

// ── Play state ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaplayerState {
    Playing,
    Paused,
    Stopped,
}

impl MediaplayerState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Playing => "playing",
            Self::Paused => "paused",
            Self::Stopped => "stopped",
        }
    }
}

impl fmt::Display for MediaplayerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Control state (tri-state for shuffle/repeat) ────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlState {
    On,
    Off,
    Disabled,
}

impl ControlState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
            Self::Disabled => "disabled",
        }
    }
}

impl fmt::Display for ControlState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ControlState {
    type Err = Homie5ProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            "disabled" => Ok(Self::Disabled),
            _ => Err(Homie5ProtocolError::InvalidPayload),
        }
    }
}

const CONTROL_STATE_FORMAT: [&str; 3] = ["on", "off", "disabled"];

// ── Set events ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum MediaplayerNodeSetEvents {
    Action(MediaplayerAction),
    Shuffle(ControlState),
    Repeat(ControlState),
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaplayerNodeConfig {
    pub next: bool,
    pub previous: bool,
    pub forward: bool,
    pub rewind: bool,
    pub stop: bool,
    pub shuffle: bool,
    pub repeat: bool,
}

impl Default for MediaplayerNodeConfig {
    fn default() -> Self {
        Self {
            next: true,
            previous: true,
            forward: true,
            rewind: true,
            stop: true,
            shuffle: true,
            repeat: true,
        }
    }
}

impl MediaplayerNodeConfig {
    fn build_action_format(&self) -> Vec<String> {
        let mut actions = vec!["play".to_owned(), "pause".to_owned()];
        if self.stop {
            actions.push("stop".to_owned());
        }
        if self.next {
            actions.push("next".to_owned());
        }
        if self.previous {
            actions.push("previous".to_owned());
        }
        if self.forward {
            actions.push("forward".to_owned());
        }
        if self.rewind {
            actions.push("rewind".to_owned());
        }
        actions
    }
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct MediaplayerNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl MediaplayerNodeBuilder {
    pub fn new(config: &MediaplayerNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(MEDIAPLAYER_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_MEDIAPLAYER);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &MediaplayerNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            MEDIAPLAYER_NODE_ACTION_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Player action")
                .format(HomiePropertyFormat::Enum(config.build_action_format()))
                .settable(true)
                .retained(false)
                .build(),
        )
        .add_property(
            MEDIAPLAYER_NODE_STATE_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Play state")
                .format(HomiePropertyFormat::Enum(vec![
                    "playing".to_owned(),
                    "paused".to_owned(),
                    "stopped".to_owned(),
                ]))
                .settable(false)
                .retained(true)
                .build(),
        )
        .add_property_cond(MEDIAPLAYER_NODE_SHUFFLE_PROP_ID, config.shuffle, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Shuffle mode")
                .format(HomiePropertyFormat::Enum(
                    CONTROL_STATE_FORMAT
                        .iter()
                        .map(|s| (*s).to_owned())
                        .collect(),
                ))
                .settable(true)
                .retained(true)
                .build()
        })
        .add_property_cond(MEDIAPLAYER_NODE_REPEAT_PROP_ID, config.repeat, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Repeat mode")
                .format(HomiePropertyFormat::Enum(
                    CONTROL_STATE_FORMAT
                        .iter()
                        .map(|s| (*s).to_owned())
                        .collect(),
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
    ) -> (HomieNodeDescription, MediaplayerNodePublisher) {
        (
            self.node_builder.build(),
            MediaplayerNodePublisher::new(
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
pub struct MediaplayerNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    action_prop: HomieID,
    state_prop: HomieID,
    shuffle_prop: HomieID,
    repeat_prop: HomieID,
}

impl MediaplayerNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            action_prop: MEDIAPLAYER_NODE_ACTION_PROP_ID,
            state_prop: MEDIAPLAYER_NODE_STATE_PROP_ID,
            shuffle_prop: MEDIAPLAYER_NODE_SHUFFLE_PROP_ID,
            repeat_prop: MEDIAPLAYER_NODE_REPEAT_PROP_ID,
        }
    }

    pub fn state(&self, value: MediaplayerState) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.state_prop, value.as_str(), true)
    }

    pub fn shuffle(&self, value: ControlState) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.shuffle_prop,
            value.as_str(),
            true,
        )
    }

    pub fn repeat(&self, value: ControlState) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.repeat_prop, value.as_str(), true)
    }
}

impl SetCommandParser for MediaplayerNodePublisher {
    type Event = MediaplayerNodeSetEvents;

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
                Ok(HomieValue::Enum(value)) => match MediaplayerAction::from_str(&value) {
                    Ok(action) => ParseOutcome::Parsed(MediaplayerNodeSetEvents::Action(action)),
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
        } else if property.match_with_node(&self.node, &self.shuffle_prop) {
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
                Ok(HomieValue::Enum(value)) => match ControlState::from_str(&value) {
                    Ok(state) => ParseOutcome::Parsed(MediaplayerNodeSetEvents::Shuffle(state)),
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
        } else if property.match_with_node(&self.node, &self.repeat_prop) {
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
                Ok(HomieValue::Enum(value)) => match ControlState::from_str(&value) {
                    Ok(state) => ParseOutcome::Parsed(MediaplayerNodeSetEvents::Repeat(state)),
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

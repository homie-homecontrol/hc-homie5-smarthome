use core::fmt;
use std::str::FromStr;

use chrono::prelude::*;

use homie5::{
    Homie5DeviceProtocol, Homie5Message, Homie5ProtocolError, HomieID, HomieValue, NodeRef,
    PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat, IntegerRange,
        NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_CAP_TIMER, SetCommandParser};

pub const TIMER_NODE_DEFAULT_ID: HomieID = HomieID::new_const("timer");
pub const TIMER_NODE_DEFAULT_NAME: &str = "Timer";
pub const TIMER_NODE_STATE_PROP_ID: HomieID = HomieID::new_const("state");
pub const TIMER_NODE_ACTION_PROP_ID: HomieID = HomieID::new_const("action");
pub const TIMER_NODE_LABEL_PROP_ID: HomieID = HomieID::new_const("label");
pub const TIMER_NODE_DURATION_PROP_ID: HomieID = HomieID::new_const("duration");
pub const TIMER_NODE_REMAINING_PROP_ID: HomieID = HomieID::new_const("remaining");
pub const TIMER_NODE_TRIGGER_TIME_PROP_ID: HomieID = HomieID::new_const("trigger-time");
pub const TIMER_NODE_CREATED_PROP_ID: HomieID = HomieID::new_const("created");

// ── Timer state ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Running,
    Paused,
    Fired,
    Cancelled,
}

impl TimerState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Fired => "fired",
            Self::Cancelled => "cancelled",
        }
    }

    pub const ALL: [TimerState; 4] = [
        TimerState::Running,
        TimerState::Paused,
        TimerState::Fired,
        TimerState::Cancelled,
    ];
}

impl fmt::Display for TimerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Timer action ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerAction {
    Start,
    Pause,
    Resume,
    Cancel,
}

impl TimerAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Cancel => "cancel",
        }
    }

    pub const ALL: [TimerAction; 4] = [
        TimerAction::Start,
        TimerAction::Pause,
        TimerAction::Resume,
        TimerAction::Cancel,
    ];
}

impl fmt::Display for TimerAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TimerAction {
    type Err = Homie5ProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "start" => Ok(Self::Start),
            "pause" => Ok(Self::Pause),
            "resume" => Ok(Self::Resume),
            "cancel" => Ok(Self::Cancel),
            _ => Err(Homie5ProtocolError::InvalidPayload),
        }
    }
}

// ── Node (state) ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct TimerNode {
    pub publisher: TimerNodePublisher,
    pub state: TimerState,
    pub label: Option<String>,
    pub duration: i64,
    pub remaining: Option<i64>,
    pub trigger_time: Option<DateTime<Utc>>,
    pub created: Option<DateTime<Utc>>,
}

// ── Set events ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum TimerNodeSetEvents {
    Action(TimerAction),
    Label(String),
    Duration(i64),
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TimerNodeConfig {
    pub label: bool,
    pub remaining: bool,
    pub trigger_time: bool,
    pub created: bool,
}

impl Default for TimerNodeConfig {
    fn default() -> Self {
        Self {
            label: true,
            remaining: true,
            trigger_time: true,
            created: true,
        }
    }
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct TimerNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl TimerNodeBuilder {
    pub fn new(config: &TimerNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(TIMER_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_TIMER);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &TimerNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            TIMER_NODE_STATE_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Timer state")
                .format(HomiePropertyFormat::Enum(
                    TimerState::ALL
                        .iter()
                        .map(|s| s.as_str().to_owned())
                        .collect(),
                ))
                .settable(false)
                .retained(true)
                .build(),
        )
        .add_property(
            TIMER_NODE_ACTION_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Timer action")
                .format(HomiePropertyFormat::Enum(
                    TimerAction::ALL
                        .iter()
                        .map(|a| a.as_str().to_owned())
                        .collect(),
                ))
                .settable(true)
                .retained(false)
                .build(),
        )
        .add_property(
            TIMER_NODE_DURATION_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Duration")
                .unit("s")
                .format(HomiePropertyFormat::IntegerRange(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                }))
                .settable(true)
                .retained(true)
                .build(),
        )
        .add_property_cond(TIMER_NODE_LABEL_PROP_ID, config.label, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::String)
                .name("Label")
                .settable(true)
                .retained(true)
                .build()
        })
        .add_property_cond(TIMER_NODE_REMAINING_PROP_ID, config.remaining, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Remaining time")
                .unit("s")
                .format(HomiePropertyFormat::IntegerRange(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                }))
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(TIMER_NODE_TRIGGER_TIME_PROP_ID, config.trigger_time, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Datetime)
                .name("Trigger time")
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(TIMER_NODE_CREATED_PROP_ID, config.created, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Datetime)
                .name("Created")
                .settable(false)
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
    ) -> (HomieNodeDescription, TimerNodePublisher) {
        (
            self.node_builder.build(),
            TimerNodePublisher::new(
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
pub struct TimerNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    state_prop: HomieID,
    action_prop: HomieID,
    label_prop: HomieID,
    duration_prop: HomieID,
    remaining_prop: HomieID,
    trigger_time_prop: HomieID,
    created_prop: HomieID,
}

impl TimerNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            state_prop: TIMER_NODE_STATE_PROP_ID,
            action_prop: TIMER_NODE_ACTION_PROP_ID,
            label_prop: TIMER_NODE_LABEL_PROP_ID,
            duration_prop: TIMER_NODE_DURATION_PROP_ID,
            remaining_prop: TIMER_NODE_REMAINING_PROP_ID,
            trigger_time_prop: TIMER_NODE_TRIGGER_TIME_PROP_ID,
            created_prop: TIMER_NODE_CREATED_PROP_ID,
        }
    }

    pub fn state(&self, value: TimerState) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.state_prop, value.as_str(), true)
    }

    pub fn label(&self, value: &str) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.label_prop, value, true)
    }

    pub fn duration(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.duration_prop,
            value.to_string(),
            true,
        )
    }

    pub fn remaining(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.remaining_prop,
            value.to_string(),
            true,
        )
    }

    pub fn trigger_time(&self, value: DateTime<Utc>) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.trigger_time_prop,
            HomieValue::DateTime(value),
            true,
        )
    }

    pub fn created(&self, value: DateTime<Utc>) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.created_prop,
            HomieValue::DateTime(value),
            true,
        )
    }
}

impl SetCommandParser for TimerNodePublisher {
    type Event = TimerNodeSetEvents;

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
                Ok(HomieValue::Enum(value)) => match TimerAction::from_str(&value) {
                    Ok(action) => ParseOutcome::Parsed(TimerNodeSetEvents::Action(action)),
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
        } else if property.match_with_node(&self.node, &self.label_prop) {
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
                Ok(HomieValue::String(value)) => {
                    ParseOutcome::Parsed(TimerNodeSetEvents::Label(value))
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
                    ParseOutcome::Parsed(TimerNodeSetEvents::Duration(value))
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
                self.action_prop.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

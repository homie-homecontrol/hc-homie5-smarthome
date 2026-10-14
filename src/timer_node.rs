use core::fmt;
use std::str::FromStr;

use chrono::prelude::*;

use homie5::{
    Homie5DeviceProtocol, Homie5Message, Homie5ProtocolError, HomieID, HomieValue, NodeRef,
    PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, IntegerRange, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, PropSpec, SMARTHOME_CAP_TIMER, SetCommandParser,
};

pub const TIMER_NODE_DEFAULT_ID: HomieID = HomieID::new_const("timer");
pub const TIMER_NODE_DEFAULT_NAME: &str = "Timer";

pub const TIMER_NODE_STATE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("state"),
    retained: true,
    settable: false,
};
pub const TIMER_NODE_ACTION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("action"),
    retained: false,
    settable: true,
};
pub const TIMER_NODE_LABEL_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("label"),
    retained: true,
    settable: true,
};
pub const TIMER_NODE_DURATION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("duration"),
    retained: true,
    settable: true,
};
pub const TIMER_NODE_REMAINING_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("remaining"),
    retained: true,
    settable: false,
};
pub const TIMER_NODE_TRIGGER_TIME_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("trigger-time"),
    retained: true,
    settable: false,
};
pub const TIMER_NODE_CREATED_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("created"),
    retained: true,
    settable: false,
};

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
            TIMER_NODE_STATE_PROP.id,
            PropertyDescriptionBuilder::enumeration(TimerState::ALL.iter().map(|s| s.as_str()))
                .unwrap()
                .name("Timer state")
                .settable(TIMER_NODE_STATE_PROP.settable)
                .retained(TIMER_NODE_STATE_PROP.retained)
                .build(),
        )
        .add_property(
            TIMER_NODE_ACTION_PROP.id,
            PropertyDescriptionBuilder::enumeration(TimerAction::ALL.iter().map(|a| a.as_str()))
                .unwrap()
                .name("Timer action")
                .settable(TIMER_NODE_ACTION_PROP.settable)
                .retained(TIMER_NODE_ACTION_PROP.retained)
                .build(),
        )
        .add_property(
            TIMER_NODE_DURATION_PROP.id,
            PropertyDescriptionBuilder::integer()
                .name("Duration")
                .unit("s")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                })
                .settable(TIMER_NODE_DURATION_PROP.settable)
                .retained(TIMER_NODE_DURATION_PROP.retained)
                .build(),
        )
        .add_property_cond(TIMER_NODE_LABEL_PROP.id, config.label, || {
            PropertyDescriptionBuilder::string()
                .name("Label")
                .settable(TIMER_NODE_LABEL_PROP.settable)
                .retained(TIMER_NODE_LABEL_PROP.retained)
                .build()
        })
        .add_property_cond(TIMER_NODE_REMAINING_PROP.id, config.remaining, || {
            PropertyDescriptionBuilder::integer()
                .name("Remaining time")
                .unit("s")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                })
                .settable(TIMER_NODE_REMAINING_PROP.settable)
                .retained(TIMER_NODE_REMAINING_PROP.retained)
                .build()
        })
        .add_property_cond(TIMER_NODE_TRIGGER_TIME_PROP.id, config.trigger_time, || {
            PropertyDescriptionBuilder::datetime()
                .name("Trigger time")
                .settable(TIMER_NODE_TRIGGER_TIME_PROP.settable)
                .retained(TIMER_NODE_TRIGGER_TIME_PROP.retained)
                .build()
        })
        .add_property_cond(TIMER_NODE_CREATED_PROP.id, config.created, || {
            PropertyDescriptionBuilder::datetime()
                .name("Created")
                .settable(TIMER_NODE_CREATED_PROP.settable)
                .retained(TIMER_NODE_CREATED_PROP.retained)
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
}

impl TimerNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn state(&self, value: TimerState) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &TIMER_NODE_STATE_PROP.id,
            value.as_str(),
            TIMER_NODE_STATE_PROP.retained,
        )
    }

    pub fn label(&self, value: &str) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &TIMER_NODE_LABEL_PROP.id,
            value,
            TIMER_NODE_LABEL_PROP.retained,
        )
    }

    pub fn duration(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &TIMER_NODE_DURATION_PROP.id,
            value.to_string(),
            TIMER_NODE_DURATION_PROP.retained,
        )
    }

    pub fn remaining(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &TIMER_NODE_REMAINING_PROP.id,
            value.to_string(),
            TIMER_NODE_REMAINING_PROP.retained,
        )
    }

    pub fn trigger_time(&self, value: DateTime<Utc>) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &TIMER_NODE_TRIGGER_TIME_PROP.id,
            HomieValue::DateTime(value),
            TIMER_NODE_TRIGGER_TIME_PROP.retained,
        )
    }

    pub fn created(&self, value: DateTime<Utc>) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &TIMER_NODE_CREATED_PROP.id,
            HomieValue::DateTime(value),
            TIMER_NODE_CREATED_PROP.retained,
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

        if property.match_with_node(&self.node, &TIMER_NODE_ACTION_PROP.id) {
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
        } else if property.match_with_node(&self.node, &TIMER_NODE_LABEL_PROP.id) {
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
        } else if property.match_with_node(&self.node, &TIMER_NODE_DURATION_PROP.id) {
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
                TIMER_NODE_ACTION_PROP.id.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

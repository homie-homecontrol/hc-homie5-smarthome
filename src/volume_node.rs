use std::str::FromStr;

use homie5::{
    HOMIE_UNIT_PERCENT, Homie5DeviceProtocol, Homie5Message, HomieID, HomieValue, NodeRef,
    PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat, IntegerRange,
        NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_CAP_VOLUME, SetCommandParser,
    mediaplayer_node::ControlState,
};

pub const VOLUME_NODE_DEFAULT_ID: HomieID = HomieID::new_const("volume");
pub const VOLUME_NODE_DEFAULT_NAME: &str = "Volume";
pub const VOLUME_NODE_LEVEL_PROP_ID: HomieID = HomieID::new_const("level");
pub const VOLUME_NODE_MUTE_PROP_ID: HomieID = HomieID::new_const("mute");

const CONTROL_STATE_FORMAT: [&str; 3] = ["on", "off", "disabled"];

// ── Set events ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum VolumeNodeSetEvents {
    Level(i64),
    Mute(ControlState),
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VolumeNodeConfig {
    pub mute: bool,
}

impl Default for VolumeNodeConfig {
    fn default() -> Self {
        Self { mute: true }
    }
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct VolumeNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl VolumeNodeBuilder {
    pub fn new(config: &VolumeNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(VOLUME_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_VOLUME);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &VolumeNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            VOLUME_NODE_LEVEL_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Volume level")
                .format(HomiePropertyFormat::IntegerRange(IntegerRange {
                    min: Some(0),
                    max: Some(100),
                    step: None,
                }))
                .unit(HOMIE_UNIT_PERCENT)
                .settable(true)
                .retained(true)
                .build(),
        )
        .add_property_cond(VOLUME_NODE_MUTE_PROP_ID, config.mute, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Mute")
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
    ) -> (HomieNodeDescription, VolumeNodePublisher) {
        (
            self.node_builder.build(),
            VolumeNodePublisher::new(
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
pub struct VolumeNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    level_prop: HomieID,
    mute_prop: HomieID,
}

impl VolumeNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            level_prop: VOLUME_NODE_LEVEL_PROP_ID,
            mute_prop: VOLUME_NODE_MUTE_PROP_ID,
        }
    }

    pub fn level(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.level_prop,
            value.to_string(),
            true,
        )
    }

    pub fn level_target(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &self.level_prop,
            value.to_string(),
            true,
        )
    }

    pub fn mute(&self, value: ControlState) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.mute_prop, value.as_str(), true)
    }
}

impl SetCommandParser for VolumeNodePublisher {
    type Event = VolumeNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &self.level_prop) {
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
                    ParseOutcome::Parsed(VolumeNodeSetEvents::Level(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &self.mute_prop) {
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
                    Ok(state) => ParseOutcome::Parsed(VolumeNodeSetEvents::Mute(state)),
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
                self.level_prop.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

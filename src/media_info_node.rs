use homie5::{
    Homie5DeviceProtocol, Homie5Message, HomieID, HomieValue, NodeRef, PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_CAP_MEDIA_INFO, SetCommandParser};

pub const MEDIA_INFO_NODE_DEFAULT_ID: HomieID = HomieID::new_const("media-info");
pub const MEDIA_INFO_NODE_DEFAULT_NAME: &str = "Media information";
pub const MEDIA_INFO_NODE_TITLE_PROP_ID: HomieID = HomieID::new_const("title");
pub const MEDIA_INFO_NODE_SUBTITLE_PROP_ID: HomieID = HomieID::new_const("subtitle");
pub const MEDIA_INFO_NODE_DESCRIPTION_PROP_ID: HomieID = HomieID::new_const("description");
pub const MEDIA_INFO_NODE_ARTWORK_PROP_ID: HomieID = HomieID::new_const("artwork");
pub const MEDIA_INFO_NODE_PROGRESS_PROP_ID: HomieID = HomieID::new_const("progress");
pub const MEDIA_INFO_NODE_LENGTH_PROP_ID: HomieID = HomieID::new_const("length");
pub const MEDIA_INFO_NODE_SEEKABLE_PROP_ID: HomieID = HomieID::new_const("seekable");
pub const MEDIA_INFO_NODE_METADATA_PROP_ID: HomieID = HomieID::new_const("metadata");

// ── Set events ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum MediaInfoNodeSetEvents {
    Progress(i64),
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaInfoNodeConfig {
    pub subtitle: bool,
    pub description: bool,
    pub artwork: bool,
    pub progress: bool,
    pub length: bool,
    pub seekable: bool,
    pub metadata: bool,
}

impl Default for MediaInfoNodeConfig {
    fn default() -> Self {
        Self {
            subtitle: true,
            description: false,
            artwork: true,
            progress: true,
            length: true,
            seekable: false,
            metadata: false,
        }
    }
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct MediaInfoNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl MediaInfoNodeBuilder {
    pub fn new(config: &MediaInfoNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(MEDIA_INFO_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_MEDIA_INFO);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &MediaInfoNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            MEDIA_INFO_NODE_TITLE_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::String)
                .name("Title")
                .settable(false)
                .retained(true)
                .build(),
        )
        .add_property_cond(MEDIA_INFO_NODE_SUBTITLE_PROP_ID, config.subtitle, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::String)
                .name("Subtitle")
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(
            MEDIA_INFO_NODE_DESCRIPTION_PROP_ID,
            config.description,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::String)
                    .name("Description")
                    .settable(false)
                    .retained(true)
                    .build()
            },
        )
        .add_property_cond(MEDIA_INFO_NODE_ARTWORK_PROP_ID, config.artwork, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::String)
                .name("Artwork URL")
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(MEDIA_INFO_NODE_PROGRESS_PROP_ID, config.progress, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Progress")
                .unit("s")
                .settable(true)
                .retained(true)
                .build()
        })
        .add_property_cond(MEDIA_INFO_NODE_LENGTH_PROP_ID, config.length, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Length")
                .unit("s")
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(MEDIA_INFO_NODE_SEEKABLE_PROP_ID, config.seekable, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                .name("Seekable")
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(MEDIA_INFO_NODE_METADATA_PROP_ID, config.metadata, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::JSON)
                .name("Metadata")
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
    ) -> (HomieNodeDescription, MediaInfoNodePublisher) {
        (
            self.node_builder.build(),
            MediaInfoNodePublisher::new(
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
pub struct MediaInfoNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    title_prop: HomieID,
    subtitle_prop: HomieID,
    description_prop: HomieID,
    artwork_prop: HomieID,
    progress_prop: HomieID,
    length_prop: HomieID,
    seekable_prop: HomieID,
    metadata_prop: HomieID,
}

impl MediaInfoNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            title_prop: MEDIA_INFO_NODE_TITLE_PROP_ID,
            subtitle_prop: MEDIA_INFO_NODE_SUBTITLE_PROP_ID,
            description_prop: MEDIA_INFO_NODE_DESCRIPTION_PROP_ID,
            artwork_prop: MEDIA_INFO_NODE_ARTWORK_PROP_ID,
            progress_prop: MEDIA_INFO_NODE_PROGRESS_PROP_ID,
            length_prop: MEDIA_INFO_NODE_LENGTH_PROP_ID,
            seekable_prop: MEDIA_INFO_NODE_SEEKABLE_PROP_ID,
            metadata_prop: MEDIA_INFO_NODE_METADATA_PROP_ID,
        }
    }

    pub fn title(&self, value: &str) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.title_prop, value, true)
    }

    pub fn subtitle(&self, value: &str) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.subtitle_prop, value, true)
    }

    pub fn description(&self, value: &str) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.description_prop, value, true)
    }

    pub fn artwork(&self, value: &str) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.artwork_prop, value, true)
    }

    pub fn progress(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.progress_prop,
            value.to_string(),
            true,
        )
    }

    pub fn length(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.length_prop,
            value.to_string(),
            true,
        )
    }

    pub fn seekable(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.seekable_prop,
            value.to_string(),
            true,
        )
    }

    pub fn metadata(&self, value: &str) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.metadata_prop, value, true)
    }
}

impl SetCommandParser for MediaInfoNodePublisher {
    type Event = MediaInfoNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &self.progress_prop) {
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
                    ParseOutcome::Parsed(MediaInfoNodeSetEvents::Progress(value))
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
                self.progress_prop.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

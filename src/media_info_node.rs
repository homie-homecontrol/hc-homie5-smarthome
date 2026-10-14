use homie5::{
    Homie5DeviceProtocol, Homie5Message, HomieID, HomieValue, NodeRef, PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, PropSpec, SMARTHOME_CAP_MEDIA_INFO, SetCommandParser,
};

pub const MEDIA_INFO_NODE_DEFAULT_ID: HomieID = HomieID::new_const("media-info");
pub const MEDIA_INFO_NODE_DEFAULT_NAME: &str = "Media information";

pub const MEDIA_INFO_NODE_TITLE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("title"),
    retained: true,
    settable: false,
};
pub const MEDIA_INFO_NODE_SUBTITLE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("subtitle"),
    retained: true,
    settable: false,
};
pub const MEDIA_INFO_NODE_DESCRIPTION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("description"),
    retained: true,
    settable: false,
};
pub const MEDIA_INFO_NODE_ARTWORK_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("artwork"),
    retained: true,
    settable: false,
};
pub const MEDIA_INFO_NODE_PROGRESS_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("progress"),
    retained: true,
    settable: true,
};
pub const MEDIA_INFO_NODE_LENGTH_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("length"),
    retained: true,
    settable: false,
};
pub const MEDIA_INFO_NODE_SEEKABLE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("seekable"),
    retained: true,
    settable: false,
};
pub const MEDIA_INFO_NODE_METADATA_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("metadata"),
    retained: true,
    settable: false,
};

// ── Node (state) ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct MediaInfoNode {
    pub publisher: MediaInfoNodePublisher,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub artwork: Option<String>,
    pub progress: Option<i64>,
    pub length: Option<i64>,
    pub seekable: Option<bool>,
    pub metadata: Option<String>,
}

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
            MEDIA_INFO_NODE_TITLE_PROP.id,
            PropertyDescriptionBuilder::string()
                .name("Title")
                .settable(MEDIA_INFO_NODE_TITLE_PROP.settable)
                .retained(MEDIA_INFO_NODE_TITLE_PROP.retained)
                .build(),
        )
        .add_property_cond(MEDIA_INFO_NODE_SUBTITLE_PROP.id, config.subtitle, || {
            PropertyDescriptionBuilder::string()
                .name("Subtitle")
                .settable(MEDIA_INFO_NODE_SUBTITLE_PROP.settable)
                .retained(MEDIA_INFO_NODE_SUBTITLE_PROP.retained)
                .build()
        })
        .add_property_cond(
            MEDIA_INFO_NODE_DESCRIPTION_PROP.id,
            config.description,
            || {
                PropertyDescriptionBuilder::string()
                    .name("Description")
                    .settable(MEDIA_INFO_NODE_DESCRIPTION_PROP.settable)
                    .retained(MEDIA_INFO_NODE_DESCRIPTION_PROP.retained)
                    .build()
            },
        )
        .add_property_cond(MEDIA_INFO_NODE_ARTWORK_PROP.id, config.artwork, || {
            PropertyDescriptionBuilder::string()
                .name("Artwork URL")
                .settable(MEDIA_INFO_NODE_ARTWORK_PROP.settable)
                .retained(MEDIA_INFO_NODE_ARTWORK_PROP.retained)
                .build()
        })
        .add_property_cond(MEDIA_INFO_NODE_PROGRESS_PROP.id, config.progress, || {
            PropertyDescriptionBuilder::integer()
                .name("Progress")
                .unit("s")
                .settable(MEDIA_INFO_NODE_PROGRESS_PROP.settable)
                .retained(MEDIA_INFO_NODE_PROGRESS_PROP.retained)
                .build()
        })
        .add_property_cond(MEDIA_INFO_NODE_LENGTH_PROP.id, config.length, || {
            PropertyDescriptionBuilder::integer()
                .name("Length")
                .unit("s")
                .settable(MEDIA_INFO_NODE_LENGTH_PROP.settable)
                .retained(MEDIA_INFO_NODE_LENGTH_PROP.retained)
                .build()
        })
        .add_property_cond(MEDIA_INFO_NODE_SEEKABLE_PROP.id, config.seekable, || {
            PropertyDescriptionBuilder::boolean()
                .name("Seekable")
                .settable(MEDIA_INFO_NODE_SEEKABLE_PROP.settable)
                .retained(MEDIA_INFO_NODE_SEEKABLE_PROP.retained)
                .build()
        })
        .add_property_cond(MEDIA_INFO_NODE_METADATA_PROP.id, config.metadata, || {
            PropertyDescriptionBuilder::json()
                .name("Metadata")
                .settable(MEDIA_INFO_NODE_METADATA_PROP.settable)
                .retained(MEDIA_INFO_NODE_METADATA_PROP.retained)
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
}

impl MediaInfoNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn title(&self, value: &str) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &MEDIA_INFO_NODE_TITLE_PROP.id,
            value,
            MEDIA_INFO_NODE_TITLE_PROP.retained,
        )
    }

    pub fn subtitle(&self, value: &str) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &MEDIA_INFO_NODE_SUBTITLE_PROP.id,
            value,
            MEDIA_INFO_NODE_SUBTITLE_PROP.retained,
        )
    }

    pub fn description(&self, value: &str) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &MEDIA_INFO_NODE_DESCRIPTION_PROP.id,
            value,
            MEDIA_INFO_NODE_DESCRIPTION_PROP.retained,
        )
    }

    pub fn artwork(&self, value: &str) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &MEDIA_INFO_NODE_ARTWORK_PROP.id,
            value,
            MEDIA_INFO_NODE_ARTWORK_PROP.retained,
        )
    }

    pub fn progress(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &MEDIA_INFO_NODE_PROGRESS_PROP.id,
            value.to_string(),
            MEDIA_INFO_NODE_PROGRESS_PROP.retained,
        )
    }

    pub fn length(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &MEDIA_INFO_NODE_LENGTH_PROP.id,
            value.to_string(),
            MEDIA_INFO_NODE_LENGTH_PROP.retained,
        )
    }

    pub fn seekable(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &MEDIA_INFO_NODE_SEEKABLE_PROP.id,
            value.to_string(),
            MEDIA_INFO_NODE_SEEKABLE_PROP.retained,
        )
    }

    pub fn metadata(&self, value: &str) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &MEDIA_INFO_NODE_METADATA_PROP.id,
            value,
            MEDIA_INFO_NODE_METADATA_PROP.retained,
        )
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

        if property.match_with_node(&self.node, &MEDIA_INFO_NODE_PROGRESS_PROP.id) {
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
                MEDIA_INFO_NODE_PROGRESS_PROP.id.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

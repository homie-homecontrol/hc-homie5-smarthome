use chrono::prelude::*;

use homie5::{
    Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, IntegerRange, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{PropSpec, SMARTHOME_CAP_LINK};

pub const LINK_NODE_DEFAULT_ID: HomieID = HomieID::new_const("link");
pub const LINK_NODE_DEFAULT_NAME: &str = "Link quality";

pub const LINK_NODE_SIGNAL_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("signal"),
    retained: true,
    settable: false,
};
pub const LINK_NODE_QUALITY_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("quality"),
    retained: true,
    settable: false,
};
pub const LINK_NODE_LAST_SEEN_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("last-seen"),
    retained: true,
    settable: false,
};

#[derive(Debug)]
pub struct LinkNode {
    pub publisher: LinkNodePublisher,
    pub signal: Option<i64>,
    pub quality: Option<i64>,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LinkNodeConfig {
    pub signal: bool,
    pub quality: bool,
    pub last_seen: bool,
}

impl Default for LinkNodeConfig {
    fn default() -> Self {
        Self {
            signal: false,
            quality: false,
            last_seen: true,
        }
    }
}

pub struct LinkNodeBuilder {
    config: LinkNodeConfig,
    node_builder: NodeDescriptionBuilder,
}

impl LinkNodeBuilder {
    pub fn new(config: &LinkNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(LINK_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_LINK);

        Self {
            node_builder: db,
            config: config.clone(),
        }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &LinkNodeConfig) -> NodeDescriptionBuilder {
        db.add_property_cond(LINK_NODE_SIGNAL_PROP.id, config.signal, || {
            PropertyDescriptionBuilder::integer()
                .name("Signal strength")
                .unit("dBm")
                .settable(LINK_NODE_SIGNAL_PROP.settable)
                .retained(LINK_NODE_SIGNAL_PROP.retained)
                .build()
        })
        .add_property_cond(LINK_NODE_QUALITY_PROP.id, config.quality, || {
            PropertyDescriptionBuilder::integer()
                .name("Link quality")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: Some(255),
                    step: None,
                })
                .settable(LINK_NODE_QUALITY_PROP.settable)
                .retained(LINK_NODE_QUALITY_PROP.retained)
                .build()
        })
        .add_property_cond(LINK_NODE_LAST_SEEN_PROP.id, config.last_seen, || {
            PropertyDescriptionBuilder::datetime()
                .name("Last seen")
                .settable(LINK_NODE_LAST_SEEN_PROP.settable)
                .retained(LINK_NODE_LAST_SEEN_PROP.retained)
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
    ) -> (HomieNodeDescription, LinkNodePublisher) {
        (
            self.node_builder.build(),
            LinkNodePublisher::new(
                NodeRef::new(
                    client.homie_domain().to_owned(),
                    client.id().to_owned(),
                    node_id,
                ),
                client.clone(),
                self.config,
            ),
        )
    }
}

#[derive(Debug)]
pub struct LinkNodePublisher {
    client: Homie5DeviceProtocol,
    config: LinkNodeConfig,
    node: NodeRef,
}

impl LinkNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol, config: LinkNodeConfig) -> Self {
        Self {
            node,
            client,
            config,
        }
    }

    pub fn signal(&self, value: i64) -> Option<homie5::client::Publish> {
        if !self.config.signal {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &LINK_NODE_SIGNAL_PROP.id,
            value.to_string(),
            LINK_NODE_SIGNAL_PROP.retained,
        ))
    }

    pub fn quality(&self, value: i64) -> Option<homie5::client::Publish> {
        if !self.config.quality {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &LINK_NODE_QUALITY_PROP.id,
            value.to_string(),
            LINK_NODE_QUALITY_PROP.retained,
        ))
    }

    pub fn last_seen(&self, value: DateTime<Utc>) -> Option<homie5::client::Publish> {
        if !self.config.last_seen {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &LINK_NODE_LAST_SEEN_PROP.id,
            value.to_rfc3339_opts(SecondsFormat::Millis, true),
            LINK_NODE_LAST_SEEN_PROP.retained,
        ))
    }
}

use core::fmt;

use chrono::prelude::*;

use homie5::{
    Homie5DeviceProtocol, HomieID, HomieValue, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_CAP_DAYLIGHT;

pub const DAYLIGHT_NODE_DEFAULT_ID: HomieID = HomieID::new_const("daylight");
pub const DAYLIGHT_NODE_DEFAULT_NAME: &str = "Daylight sensor";
pub const DAYLIGHT_NODE_DAYLIGHT_PROP_ID: HomieID = HomieID::new_const("daylight");
pub const DAYLIGHT_NODE_DARK_PROP_ID: HomieID = HomieID::new_const("dark");
pub const DAYLIGHT_NODE_SUNRISE_PROP_ID: HomieID = HomieID::new_const("sunrise");
pub const DAYLIGHT_NODE_SUNSET_PROP_ID: HomieID = HomieID::new_const("sunset");
pub const DAYLIGHT_NODE_PHASE_PROP_ID: HomieID = HomieID::new_const("phase");

// ── Daylight phase ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaylightPhase {
    Night,
    Dawn,
    Morning,
    Day,
    Evening,
    Dusk,
}

impl DaylightPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Night => "night",
            Self::Dawn => "dawn",
            Self::Morning => "morning",
            Self::Day => "day",
            Self::Evening => "evening",
            Self::Dusk => "dusk",
        }
    }

    pub const ALL: [DaylightPhase; 6] = [
        DaylightPhase::Night,
        DaylightPhase::Dawn,
        DaylightPhase::Morning,
        DaylightPhase::Day,
        DaylightPhase::Evening,
        DaylightPhase::Dusk,
    ];
}

impl fmt::Display for DaylightPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Node (state) ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct DaylightNode {
    pub publisher: DaylightNodePublisher,
    pub daylight: bool,
    pub dark: bool,
    pub sunrise: DateTime<Utc>,
    pub sunset: DateTime<Utc>,
    pub phase: Option<DaylightPhase>,
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaylightNodeConfig {
    pub phase: bool,
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct DaylightNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl DaylightNodeBuilder {
    pub fn new(config: &DaylightNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(DAYLIGHT_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_DAYLIGHT);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &DaylightNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            DAYLIGHT_NODE_DAYLIGHT_PROP_ID,
            PropertyDescriptionBuilder::boolean()
                .name("Daylight")
                .boolean_labels("night", "day")
                .settable(false)
                .retained(true)
                .build(),
        )
        .add_property(
            DAYLIGHT_NODE_DARK_PROP_ID,
            PropertyDescriptionBuilder::boolean()
                .name("Dark")
                .boolean_labels("light", "dark")
                .settable(false)
                .retained(true)
                .build(),
        )
        .add_property(
            DAYLIGHT_NODE_SUNRISE_PROP_ID,
            PropertyDescriptionBuilder::datetime()
                .name("Sunrise")
                .settable(false)
                .retained(true)
                .build(),
        )
        .add_property(
            DAYLIGHT_NODE_SUNSET_PROP_ID,
            PropertyDescriptionBuilder::datetime()
                .name("Sunset")
                .settable(false)
                .retained(true)
                .build(),
        )
        .add_property_cond(DAYLIGHT_NODE_PHASE_PROP_ID, config.phase, || {
            PropertyDescriptionBuilder::enumeration(
                DaylightPhase::ALL.iter().map(|p| p.as_str()),
            )
            .unwrap()
            .name("Daylight phase")
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
    ) -> (HomieNodeDescription, DaylightNodePublisher) {
        (
            self.node_builder.build(),
            DaylightNodePublisher::new(
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
pub struct DaylightNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    daylight_prop: HomieID,
    dark_prop: HomieID,
    sunrise_prop: HomieID,
    sunset_prop: HomieID,
    phase_prop: HomieID,
}

impl DaylightNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            daylight_prop: DAYLIGHT_NODE_DAYLIGHT_PROP_ID,
            dark_prop: DAYLIGHT_NODE_DARK_PROP_ID,
            sunrise_prop: DAYLIGHT_NODE_SUNRISE_PROP_ID,
            sunset_prop: DAYLIGHT_NODE_SUNSET_PROP_ID,
            phase_prop: DAYLIGHT_NODE_PHASE_PROP_ID,
        }
    }

    pub fn daylight(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.daylight_prop,
            value.to_string(),
            true,
        )
    }

    pub fn dark(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.dark_prop,
            value.to_string(),
            true,
        )
    }

    pub fn sunrise(&self, value: DateTime<Utc>) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.sunrise_prop,
            HomieValue::DateTime(value),
            true,
        )
    }

    pub fn sunset(&self, value: DateTime<Utc>) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.sunset_prop,
            HomieValue::DateTime(value),
            true,
        )
    }

    pub fn phase(&self, value: DaylightPhase) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.phase_prop, value.as_str(), true)
    }
}

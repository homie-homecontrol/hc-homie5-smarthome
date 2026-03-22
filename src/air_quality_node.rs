use homie5::{
    Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, IntegerRange, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_CAP_AIR_QUALITY;

pub const AIR_QUALITY_NODE_DEFAULT_ID: HomieID = HomieID::new_const("air-quality");
pub const AIR_QUALITY_NODE_DEFAULT_NAME: &str = "Air quality";
pub const AIR_QUALITY_NODE_CO2_PROP_ID: HomieID = HomieID::new_const("co2");
pub const AIR_QUALITY_NODE_VOC_PROP_ID: HomieID = HomieID::new_const("voc");
pub const AIR_QUALITY_NODE_PM25_PROP_ID: HomieID = HomieID::new_const("pm25");
pub const AIR_QUALITY_NODE_PM10_PROP_ID: HomieID = HomieID::new_const("pm10");
pub const AIR_QUALITY_NODE_AQI_PROP_ID: HomieID = HomieID::new_const("aqi");

// ── Node (state) ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct AirQualityNode {
    pub publisher: AirQualityNodePublisher,
    pub co2: Option<i64>,
    pub voc: Option<i64>,
    pub pm25: Option<i64>,
    pub pm10: Option<i64>,
    pub aqi: Option<i64>,
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AirQualityNodeConfig {
    pub co2: bool,
    pub voc: bool,
    pub pm25: bool,
    pub pm10: bool,
    pub aqi: bool,
}

impl Default for AirQualityNodeConfig {
    fn default() -> Self {
        Self {
            co2: true,
            voc: false,
            pm25: false,
            pm10: false,
            aqi: false,
        }
    }
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct AirQualityNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl AirQualityNodeBuilder {
    pub fn new(config: &AirQualityNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(AIR_QUALITY_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_AIR_QUALITY);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &AirQualityNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property_cond(AIR_QUALITY_NODE_CO2_PROP_ID, config.co2, || {
            PropertyDescriptionBuilder::integer()
                .name("CO₂")
                .unit("ppm")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                })
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(AIR_QUALITY_NODE_VOC_PROP_ID, config.voc, || {
            PropertyDescriptionBuilder::integer()
                .name("VOC")
                .unit("ppb")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                })
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(AIR_QUALITY_NODE_PM25_PROP_ID, config.pm25, || {
            PropertyDescriptionBuilder::integer()
                .name("PM2.5")
                .unit("µg/m³")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                })
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(AIR_QUALITY_NODE_PM10_PROP_ID, config.pm10, || {
            PropertyDescriptionBuilder::integer()
                .name("PM10")
                .unit("µg/m³")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                })
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(AIR_QUALITY_NODE_AQI_PROP_ID, config.aqi, || {
            PropertyDescriptionBuilder::integer()
                .name("Air quality index")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: Some(500),
                    step: None,
                })
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
    ) -> (HomieNodeDescription, AirQualityNodePublisher) {
        (
            self.node_builder.build(),
            AirQualityNodePublisher::new(
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
pub struct AirQualityNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    co2_prop: HomieID,
    voc_prop: HomieID,
    pm25_prop: HomieID,
    pm10_prop: HomieID,
    aqi_prop: HomieID,
}

impl AirQualityNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            co2_prop: AIR_QUALITY_NODE_CO2_PROP_ID,
            voc_prop: AIR_QUALITY_NODE_VOC_PROP_ID,
            pm25_prop: AIR_QUALITY_NODE_PM25_PROP_ID,
            pm10_prop: AIR_QUALITY_NODE_PM10_PROP_ID,
            aqi_prop: AIR_QUALITY_NODE_AQI_PROP_ID,
        }
    }

    pub fn co2(&self, value: i64) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.co2_prop, value.to_string(), true)
    }

    pub fn voc(&self, value: i64) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.voc_prop, value.to_string(), true)
    }

    pub fn pm25(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.pm25_prop,
            value.to_string(),
            true,
        )
    }

    pub fn pm10(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.pm10_prop,
            value.to_string(),
            true,
        )
    }

    pub fn aqi(&self, value: i64) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.aqi_prop, value.to_string(), true)
    }
}

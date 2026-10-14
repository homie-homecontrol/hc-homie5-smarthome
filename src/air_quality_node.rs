use homie5::{
    Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, IntegerRange, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{PropSpec, SMARTHOME_CAP_AIR_QUALITY};

pub const AIR_QUALITY_NODE_DEFAULT_ID: HomieID = HomieID::new_const("air-quality");
pub const AIR_QUALITY_NODE_DEFAULT_NAME: &str = "Air quality";

pub const AIR_QUALITY_NODE_CO2_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("co2"),
    retained: true,
    settable: false,
};
pub const AIR_QUALITY_NODE_VOC_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("voc"),
    retained: true,
    settable: false,
};
pub const AIR_QUALITY_NODE_PM25_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("pm25"),
    retained: true,
    settable: false,
};
pub const AIR_QUALITY_NODE_PM10_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("pm10"),
    retained: true,
    settable: false,
};
pub const AIR_QUALITY_NODE_AQI_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("aqi"),
    retained: true,
    settable: false,
};

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
        db.add_property_cond(AIR_QUALITY_NODE_CO2_PROP.id, config.co2, || {
            PropertyDescriptionBuilder::integer()
                .name("CO₂")
                .unit("ppm")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                })
                .settable(AIR_QUALITY_NODE_CO2_PROP.settable)
                .retained(AIR_QUALITY_NODE_CO2_PROP.retained)
                .build()
        })
        .add_property_cond(AIR_QUALITY_NODE_VOC_PROP.id, config.voc, || {
            PropertyDescriptionBuilder::integer()
                .name("VOC")
                .unit("ppb")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                })
                .settable(AIR_QUALITY_NODE_VOC_PROP.settable)
                .retained(AIR_QUALITY_NODE_VOC_PROP.retained)
                .build()
        })
        .add_property_cond(AIR_QUALITY_NODE_PM25_PROP.id, config.pm25, || {
            PropertyDescriptionBuilder::integer()
                .name("PM2.5")
                .unit("µg/m³")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                })
                .settable(AIR_QUALITY_NODE_PM25_PROP.settable)
                .retained(AIR_QUALITY_NODE_PM25_PROP.retained)
                .build()
        })
        .add_property_cond(AIR_QUALITY_NODE_PM10_PROP.id, config.pm10, || {
            PropertyDescriptionBuilder::integer()
                .name("PM10")
                .unit("µg/m³")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: None,
                    step: None,
                })
                .settable(AIR_QUALITY_NODE_PM10_PROP.settable)
                .retained(AIR_QUALITY_NODE_PM10_PROP.retained)
                .build()
        })
        .add_property_cond(AIR_QUALITY_NODE_AQI_PROP.id, config.aqi, || {
            PropertyDescriptionBuilder::integer()
                .name("Air quality index")
                .integer_range(IntegerRange {
                    min: Some(0),
                    max: Some(500),
                    step: None,
                })
                .settable(AIR_QUALITY_NODE_AQI_PROP.settable)
                .retained(AIR_QUALITY_NODE_AQI_PROP.retained)
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
}

impl AirQualityNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn co2(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &AIR_QUALITY_NODE_CO2_PROP.id,
            value.to_string(),
            AIR_QUALITY_NODE_CO2_PROP.retained,
        )
    }

    pub fn voc(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &AIR_QUALITY_NODE_VOC_PROP.id,
            value.to_string(),
            AIR_QUALITY_NODE_VOC_PROP.retained,
        )
    }

    pub fn pm25(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &AIR_QUALITY_NODE_PM25_PROP.id,
            value.to_string(),
            AIR_QUALITY_NODE_PM25_PROP.retained,
        )
    }

    pub fn pm10(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &AIR_QUALITY_NODE_PM10_PROP.id,
            value.to_string(),
            AIR_QUALITY_NODE_PM10_PROP.retained,
        )
    }

    pub fn aqi(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &AIR_QUALITY_NODE_AQI_PROP.id,
            value.to_string(),
            AIR_QUALITY_NODE_AQI_PROP.retained,
        )
    }
}

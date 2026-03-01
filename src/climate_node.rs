use homie5::{
    HOMIE_UNIT_DEGREE_CELSIUS, HOMIE_UNIT_KILOPASCAL, HOMIE_UNIT_PERCENT, Homie5DeviceProtocol,
    HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_TYPE_CLIMATE;

pub const CLIMATE_NODE_DEFAULT_ID: HomieID = HomieID::new_const("climate");
pub const CLIMATE_NODE_DEFAULT_NAME: &str = "Climate sensor";
pub const CLIMATE_NODE_TEMP_PROP_ID: HomieID = HomieID::new_const("temperature");
pub const CLIMATE_NODE_HUM_PROP_ID: HomieID = HomieID::new_const("humidity");
pub const CLIMATE_NODE_PRES_PROP_ID: HomieID = HomieID::new_const("pressure");

#[derive(Debug)]
pub struct ClimateNode {
    pub publisher: ClimateNodePublisher,
    pub temperature: Option<f64>,
    pub humidity: Option<i64>,
    pub pressure: Option<f64>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClimateNodeConfig {
    pub temperature: bool,
    pub humidity: bool,
    pub pressure: bool,
    pub temp_unit: String,
}

impl Default for ClimateNodeConfig {
    fn default() -> Self {
        Self {
            temperature: true,
            humidity: true,
            pressure: false,
            temp_unit: HOMIE_UNIT_DEGREE_CELSIUS.to_owned(),
        }
    }
}

pub struct ClimateNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl ClimateNodeBuilder {
    pub fn new(config: &ClimateNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(CLIMATE_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_TYPE_CLIMATE);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &ClimateNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property_cond(CLIMATE_NODE_TEMP_PROP_ID, config.temperature, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                .name("Current temperature")
                .retained(true)
                .settable(false)
                .unit(config.temp_unit.to_owned())
                .build()
        })
        .add_property_cond(CLIMATE_NODE_HUM_PROP_ID, config.humidity, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Current humidity")
                .retained(true)
                .settable(false)
                .unit(HOMIE_UNIT_PERCENT)
                .build()
        })
        .add_property_cond(CLIMATE_NODE_PRES_PROP_ID, config.pressure, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                .name("Current pressure")
                .retained(true)
                .settable(false)
                .unit(HOMIE_UNIT_KILOPASCAL)
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
    ) -> (HomieNodeDescription, ClimateNodePublisher) {
        (
            self.node_builder.build(),
            ClimateNodePublisher::new(
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

#[derive(Debug)]
pub struct ClimateNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    temp_prop: HomieID,
    hum_prop: HomieID,
    pres_prop: HomieID,
}

impl ClimateNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            temp_prop: CLIMATE_NODE_TEMP_PROP_ID,
            hum_prop: CLIMATE_NODE_HUM_PROP_ID,
            pres_prop: CLIMATE_NODE_PRES_PROP_ID,
        }
    }

    pub fn temperature(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.temp_prop,
            value.to_string(),
            true,
        )
    }

    pub fn humidity(&self, value: i64) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.hum_prop, value.to_string(), true)
    }

    pub fn pressure(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.pres_prop,
            value.to_string(),
            true,
        )
    }
}

use homie5::{
    HOMIE_UNIT_DEGREE_CELSIUS, HOMIE_UNIT_KILOPASCAL, HOMIE_UNIT_PERCENT, Homie5DeviceProtocol,
    HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{PropSpec, SMARTHOME_CAP_CLIMATE};

pub const CLIMATE_NODE_DEFAULT_ID: HomieID = HomieID::new_const("climate");
pub const CLIMATE_NODE_DEFAULT_NAME: &str = "Climate sensor";

pub const CLIMATE_NODE_TEMP_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("temperature"),
    retained: true,
    settable: false,
};
pub const CLIMATE_NODE_HUM_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("humidity"),
    retained: true,
    settable: false,
};
pub const CLIMATE_NODE_PRES_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("pressure"),
    retained: true,
    settable: false,
};

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
        .r#type(SMARTHOME_CAP_CLIMATE);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &ClimateNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property_cond(CLIMATE_NODE_TEMP_PROP.id, config.temperature, || {
            PropertyDescriptionBuilder::float()
                .name("Current temperature")
                .retained(CLIMATE_NODE_TEMP_PROP.retained)
                .settable(CLIMATE_NODE_TEMP_PROP.settable)
                .unit(config.temp_unit.to_owned())
                .build()
        })
        .add_property_cond(CLIMATE_NODE_HUM_PROP.id, config.humidity, || {
            PropertyDescriptionBuilder::integer()
                .name("Current humidity")
                .retained(CLIMATE_NODE_HUM_PROP.retained)
                .settable(CLIMATE_NODE_HUM_PROP.settable)
                .unit(HOMIE_UNIT_PERCENT)
                .build()
        })
        .add_property_cond(CLIMATE_NODE_PRES_PROP.id, config.pressure, || {
            PropertyDescriptionBuilder::float()
                .name("Current pressure")
                .retained(CLIMATE_NODE_PRES_PROP.retained)
                .settable(CLIMATE_NODE_PRES_PROP.settable)
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
}

impl ClimateNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn temperature(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &CLIMATE_NODE_TEMP_PROP.id,
            value.to_string(),
            CLIMATE_NODE_TEMP_PROP.retained,
        )
    }

    pub fn humidity(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &CLIMATE_NODE_HUM_PROP.id,
            value.to_string(),
            CLIMATE_NODE_HUM_PROP.retained,
        )
    }

    pub fn pressure(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &CLIMATE_NODE_PRES_PROP.id,
            value.to_string(),
            CLIMATE_NODE_PRES_PROP.retained,
        )
    }
}

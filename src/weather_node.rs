use homie5::{
    HOMIE_UNIT_DEGREE_CELSIUS, HOMIE_UNIT_KILOPASCAL, HOMIE_UNIT_PERCENT, Homie5DeviceProtocol,
    HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_TYPE_WEATHER;

pub const WEATHER_NODE_DEFAULT_ID: &str = "weather";
pub const WEATHER_NODE_DEFAULT_NAME: &str = "Weather clima sensor";
pub const WEATHER_NODE_TEMP_PROP_ID: &str = "temperature";
pub const WEATHER_NODE_HUM_PROP_ID: &str = "humidity";
pub const WEATHER_NODE_PRES_PROP_ID: &str = "pressure";

#[derive(Debug)]
pub struct WeatherNode {
    pub publisher: WeatherNodePublisher,
    pub temperature: Option<f64>,
    pub humidity: Option<i64>,
    pub pressure: Option<f64>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WeatherNodeConfig {
    pub temperature: bool,
    pub humidity: bool,
    pub pressure: bool,
    pub temp_unit: String,
}

impl Default for WeatherNodeConfig {
    fn default() -> Self {
        Self {
            temperature: true,
            humidity: true,
            pressure: false,
            temp_unit: HOMIE_UNIT_DEGREE_CELSIUS.to_owned(),
        }
    }
}

pub struct WeatherNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl WeatherNodeBuilder {
    pub fn new(config: &WeatherNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(WEATHER_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_TYPE_WEATHER);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &WeatherNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property_cond(
            WEATHER_NODE_TEMP_PROP_ID.try_into().unwrap(),
            config.temperature,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                    .name("Current temperature")
                    .retained(true)
                    .settable(false)
                    .unit(config.temp_unit.to_owned())
                    .build()
            },
        )
        .add_property_cond(
            WEATHER_NODE_HUM_PROP_ID.try_into().unwrap(),
            config.humidity,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                    .name("Current humidity")
                    .retained(true)
                    .settable(false)
                    .unit(HOMIE_UNIT_PERCENT)
                    .build()
            },
        )
        .add_property_cond(
            WEATHER_NODE_PRES_PROP_ID.try_into().unwrap(),
            config.pressure,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                    .name("Current pressure")
                    .retained(true)
                    .settable(false)
                    .unit(HOMIE_UNIT_KILOPASCAL)
                    .build()
            },
        )
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
    ) -> (HomieNodeDescription, WeatherNodePublisher) {
        (
            self.node_builder.build(),
            WeatherNodePublisher::new(
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
pub struct WeatherNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    temp_prop: HomieID,
    hum_prop: HomieID,
    pres_prop: HomieID,
}

impl WeatherNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            temp_prop: WEATHER_NODE_TEMP_PROP_ID.try_into().unwrap(),
            hum_prop: WEATHER_NODE_HUM_PROP_ID.try_into().unwrap(),
            pres_prop: WEATHER_NODE_PRES_PROP_ID.try_into().unwrap(),
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

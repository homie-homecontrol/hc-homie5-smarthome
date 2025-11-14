use homie5::{
    HOMIE_UNIT_HERTZ, HOMIE_UNIT_MILI_AMPERE, HOMIE_UNIT_VOLT, HOMIE_UNIT_WATT,
    Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        FloatRange, HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_TYPE_POWERMETER;

pub const POWERMETER_NODE_DEFAULT_ID: &str = "powermeter";
pub const POWERMETER_NODE_DEFAULT_NAME: &str = "Powermeter";
pub const POWERMETER_NODE_POWER_PROP_ID: HomieID = HomieID::new_const("power");
pub const POWERMETER_NODE_CURRENT_PROP_ID: HomieID = HomieID::new_const("current");
pub const POWERMETER_NODE_VOLTAGE_PROP_ID: HomieID = HomieID::new_const("voltage");
pub const POWERMETER_NODE_FREQUENCY_PROP_ID: HomieID = HomieID::new_const("frequency");
pub const POWERMETER_NODE_CONSUMPTION_PROP_ID: HomieID = HomieID::new_const("consumption");

#[derive(Debug)]
pub struct PowermeterNode {
    pub publisher: PowermeterNodePublisher,
    pub power: f64,
    pub current: Option<f64>,
    pub voltage: Option<f64>,
    pub frequency: Option<f64>,
    pub consumption: Option<f64>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PowermeterNodeConfig {
    pub current: bool,
    pub voltage: bool,
    pub frequency: bool,
    pub consumption: bool,
}

impl Default for PowermeterNodeConfig {
    fn default() -> Self {
        Self {
            current: true,
            voltage: true,
            frequency: false,
            consumption: true,
        }
    }
}
pub struct PowermeterNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl Default for PowermeterNodeBuilder {
    fn default() -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(POWERMETER_NODE_DEFAULT_NAME),
            &Default::default(),
        )
        .r#type(SMARTHOME_TYPE_POWERMETER);

        Self { node_builder: db }
    }
}

impl PowermeterNodeBuilder {
    pub fn new(config: &PowermeterNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(POWERMETER_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_TYPE_POWERMETER);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &PowermeterNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            POWERMETER_NODE_POWER_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                .name("Power")
                .unit(HOMIE_UNIT_WATT)
                .format(HomiePropertyFormat::FloatRange(FloatRange {
                    min: Some(0.0),
                    max: None,
                    step: None,
                }))
                .settable(false)
                .retained(true)
                .build(),
        )
        .add_property_cond(POWERMETER_NODE_CURRENT_PROP_ID, config.current, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                .name("Current")
                .unit(HOMIE_UNIT_MILI_AMPERE)
                .format(HomiePropertyFormat::FloatRange(FloatRange {
                    min: Some(0.0),
                    max: None,
                    step: None,
                }))
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(POWERMETER_NODE_VOLTAGE_PROP_ID, config.voltage, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                .name("Voltage")
                .unit(HOMIE_UNIT_VOLT)
                .format(HomiePropertyFormat::FloatRange(FloatRange {
                    min: Some(0.0),
                    max: None,
                    step: None,
                }))
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(POWERMETER_NODE_FREQUENCY_PROP_ID, config.frequency, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                .name("Frequency")
                .unit(HOMIE_UNIT_HERTZ)
                .format(HomiePropertyFormat::FloatRange(FloatRange {
                    min: Some(0.0),
                    max: None,
                    step: None,
                }))
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(
            POWERMETER_NODE_CONSUMPTION_PROP_ID,
            config.consumption,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                    .name("Consumption")
                    .unit("wH") //WATT HOURS
                    .format(HomiePropertyFormat::FloatRange(FloatRange {
                        min: Some(0.0),
                        max: None,
                        step: None,
                    }))
                    .settable(false)
                    .retained(true)
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
    ) -> (HomieNodeDescription, PowermeterNodePublisher) {
        (
            self.node_builder.build(),
            PowermeterNodePublisher::new(
                NodeRef::new(
                    client.homie_domain().to_owned(),
                    client.id().to_owned(),
                    node_id,
                ),
                client.clone(),
            ),
        )
    }
}

#[derive(Debug)]
pub struct PowermeterNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    power_prop: HomieID,
    current_prop: HomieID,
    voltage_prop: HomieID,
    frequency_prop: HomieID,
    consumption_prop: HomieID,
}

impl PowermeterNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            power_prop: POWERMETER_NODE_POWER_PROP_ID,
            current_prop: POWERMETER_NODE_CURRENT_PROP_ID,
            voltage_prop: POWERMETER_NODE_VOLTAGE_PROP_ID,
            frequency_prop: POWERMETER_NODE_FREQUENCY_PROP_ID,
            consumption_prop: POWERMETER_NODE_CONSUMPTION_PROP_ID,
        }
    }

    pub fn power(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.power_prop,
            value.to_string(),
            true,
        )
    }
    pub fn current(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.current_prop,
            value.to_string(),
            true,
        )
    }
    pub fn voltage(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.voltage_prop,
            value.to_string(),
            true,
        )
    }
    pub fn frequency(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.frequency_prop,
            value.to_string(),
            true,
        )
    }
    pub fn consumption(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.consumption_prop,
            value.to_string(),
            true,
        )
    }
}

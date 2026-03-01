use homie5::{
    HOMIE_UNIT_PERCENT, Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, HomiePropertyFormat, IntegerRange, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_CAP_BATTERY;

pub const BATTERY_NODE_DEFAULT_ID: HomieID = HomieID::new_const("battery");
pub const BATTERY_NODE_DEFAULT_NAME: &str = "Battery";
pub const BATTERY_NODE_LEVEL_PROP_ID: HomieID = HomieID::new_const("level");
pub const BATTERY_NODE_VOLTAGE_PROP_ID: HomieID = HomieID::new_const("voltage");

#[derive(Debug)]
pub struct BatteryNode {
    pub publisher: BatteryNodePublisher,
    pub level: Option<i64>,
    pub voltage: Option<i64>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BatteryNodeConfig {
    pub level: bool,
    pub voltage: bool,
}

impl Default for BatteryNodeConfig {
    fn default() -> Self {
        Self {
            level: true,
            voltage: false,
        }
    }
}

pub struct BatteryNodeBuilder {
    config: BatteryNodeConfig,
    node_builder: NodeDescriptionBuilder,
}

impl BatteryNodeBuilder {
    pub fn new(config: &BatteryNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(BATTERY_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_BATTERY);

        Self {
            node_builder: db,
            config: config.clone(),
        }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &BatteryNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property_cond(BATTERY_NODE_LEVEL_PROP_ID, config.level, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Battery level")
                .format(HomiePropertyFormat::IntegerRange(IntegerRange {
                    min: Some(0),
                    max: Some(100),
                    step: None,
                }))
                .unit(HOMIE_UNIT_PERCENT)
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(BATTERY_NODE_VOLTAGE_PROP_ID, config.voltage, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Battery voltage")
                .unit("mV")
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
    ) -> (HomieNodeDescription, BatteryNodePublisher) {
        (
            self.node_builder.build(),
            BatteryNodePublisher::new(
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
pub struct BatteryNodePublisher {
    client: Homie5DeviceProtocol,
    config: BatteryNodeConfig,
    node: NodeRef,
    level_prop: HomieID,
    voltage_prop: HomieID,
}

impl BatteryNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol, config: BatteryNodeConfig) -> Self {
        Self {
            node,
            client,
            config,
            level_prop: BATTERY_NODE_LEVEL_PROP_ID,
            voltage_prop: BATTERY_NODE_VOLTAGE_PROP_ID,
        }
    }

    pub fn level(&self, value: i64) -> Option<homie5::client::Publish> {
        if !self.config.level {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &self.level_prop,
            value.to_string(),
            true,
        ))
    }

    pub fn voltage(&self, value: i64) -> Option<homie5::client::Publish> {
        if !self.config.voltage {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &self.voltage_prop,
            value.to_string(),
            true,
        ))
    }
}

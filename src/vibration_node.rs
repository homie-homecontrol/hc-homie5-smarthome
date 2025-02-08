use homie5::{
    device_description::{
        HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
    Homie5DeviceProtocol, HomieID, NodeRef, HOMIE_UNIT_LUX,
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_TYPE_VIBRATION;

pub const VIBRATION_NODE_DEFAULT_ID: &str = "vibration";
pub const VIBRATION_NODE_DEFAULT_NAME: &str = "Vibration sensor";
pub const VIBRATION_NODE_VIBRATION_PROP_ID: &str = "vibration";
pub const VIBRATION_NODE_VIBRATION_STRENGTH_PROP_ID: &str = "vibration-strength";

#[derive(Debug)]
pub struct VibrationNode {
    pub publisher: VibrationNodePublisher,
    pub vibration: bool,
    pub vibration_strength: Option<i64>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct VibrationNodeConfig {
    pub vibration_strength: bool,
}

impl Default for VibrationNodeConfig {
    fn default() -> Self {
        Self {
            vibration_strength: true,
        }
    }
}

pub struct VibrationNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl VibrationNodeBuilder {
    pub fn new(config: &VibrationNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(VIBRATION_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_TYPE_VIBRATION);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &VibrationNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            VIBRATION_NODE_VIBRATION_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                .name("Vibration detected")
                .format(HomiePropertyFormat::Boolean {
                    false_val: "no-vibration".to_owned(),
                    true_val: "vibration".to_owned(),
                })
                .retained(true)
                .settable(false)
                .build(),
        )
        .add_property_cond(
            VIBRATION_NODE_VIBRATION_STRENGTH_PROP_ID
                .try_into()
                .unwrap(),
            config.vibration_strength,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                    .name("Current lightlevel")
                    .retained(true)
                    .settable(false)
                    .unit(HOMIE_UNIT_LUX)
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
    ) -> (HomieNodeDescription, VibrationNodePublisher) {
        (
            self.node_builder.build(),
            VibrationNodePublisher::new(
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
pub struct VibrationNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    vibr_prop: HomieID,
    vibr_strength: HomieID,
}

impl VibrationNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            vibr_prop: VIBRATION_NODE_VIBRATION_PROP_ID.try_into().unwrap(),
            vibr_strength: VIBRATION_NODE_VIBRATION_STRENGTH_PROP_ID
                .try_into()
                .unwrap(),
        }
    }

    pub fn vibration(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.vibr_prop,
            value.to_string(),
            true,
        )
    }

    pub fn vibration_strength(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.vibr_strength,
            value.to_string(),
            true,
        )
    }
}

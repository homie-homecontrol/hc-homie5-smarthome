use homie5::{
    HOMIE_UNIT_LUX, Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        BooleanFormat, HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_TYPE_MOTION;

pub const MOTION_NODE_DEFAULT_ID: &str = "motion";
pub const MOTION_NODE_DEFAULT_NAME: &str = "Motion sensor";
pub const MOTION_NODE_MOTION_PROP_ID: &str = "motion";
pub const MOTION_NODE_LUX_PROP_ID: &str = "lux";

#[derive(Debug)]
pub struct MotionNode {
    pub publisher: MotionNodePublisher,
    pub motion: bool,
    pub lux: Option<i64>,
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct MotionNodeConfig {
    pub lux: bool,
}

pub struct MotionNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl MotionNodeBuilder {
    pub fn new(config: &MotionNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(MOTION_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_TYPE_MOTION);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &MotionNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            MOTION_NODE_MOTION_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                .name("Motion detected")
                .format(HomiePropertyFormat::Boolean(BooleanFormat {
                    false_val: "no-motion".to_owned(),
                    true_val: "motion".to_owned(),
                }))
                .retained(true)
                .settable(false)
                .build(),
        )
        .add_property_cond(
            MOTION_NODE_LUX_PROP_ID.try_into().unwrap(),
            config.lux,
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
    ) -> (HomieNodeDescription, MotionNodePublisher) {
        (
            self.node_builder.build(),
            MotionNodePublisher::new(
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
pub struct MotionNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    motion_prop: HomieID,
    lux_prop: HomieID,
}

impl MotionNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            motion_prop: MOTION_NODE_MOTION_PROP_ID.try_into().unwrap(),
            lux_prop: MOTION_NODE_LUX_PROP_ID.try_into().unwrap(),
        }
    }

    pub fn motion(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.motion_prop,
            value.to_string(),
            true,
        )
    }

    pub fn lux(&self, value: i64) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.lux_prop, value.to_string(), true)
    }
}

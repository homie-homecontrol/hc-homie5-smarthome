use homie5::{
    Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        BooleanFormat, HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};

use crate::SMARTHOME_CAP_MOTION;

pub const MOTION_NODE_DEFAULT_ID: HomieID = HomieID::new_const("motion");
pub const MOTION_NODE_DEFAULT_NAME: &str = "Motion sensor";
pub const MOTION_NODE_MOTION_PROP_ID: HomieID = HomieID::new_const("motion");

#[derive(Debug)]
pub struct MotionNode {
    pub publisher: MotionNodePublisher,
    pub motion: bool,
}

pub struct MotionNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl Default for MotionNodeBuilder {
    fn default() -> Self {
        let db = Self::build_node(NodeDescriptionBuilder::new().name(MOTION_NODE_DEFAULT_NAME))
            .r#type(SMARTHOME_CAP_MOTION);

        Self { node_builder: db }
    }
}

impl MotionNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            MOTION_NODE_MOTION_PROP_ID,
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
}

impl MotionNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            motion_prop: MOTION_NODE_MOTION_PROP_ID,
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
}

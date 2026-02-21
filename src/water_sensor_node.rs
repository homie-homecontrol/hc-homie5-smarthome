use homie5::{
    Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        BooleanFormat, HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};

use crate::SMARTHOME_TYPE_WATER_SENSOR;

pub const WATER_SENSOR_NODE_DEFAULT_ID: &str = "water";
pub const WATER_SENSOR_NODE_DEFAULT_NAME: &str = "Open/Close water";
pub const WATER_SENSOR_NODE_DETECTED_PROP_ID: &str = "detected";

#[derive(Debug)]
pub struct WaterSensorNode {
    pub publisher: WaterSensorNodePublisher,
    pub detected: bool,
}

pub struct WaterSensorNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl Default for WaterSensorNodeBuilder {
    fn default() -> Self {
        let db =
            Self::build_node(NodeDescriptionBuilder::new().name(WATER_SENSOR_NODE_DEFAULT_NAME))
                .r#type(SMARTHOME_TYPE_WATER_SENSOR);

        Self { node_builder: db }
    }
}

impl WaterSensorNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            WATER_SENSOR_NODE_DETECTED_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                .name("Water detection")
                .format(HomiePropertyFormat::Boolean(BooleanFormat {
                    false_val: "no water".to_owned(),
                    true_val: "water detected".to_owned(),
                }))
                .settable(false)
                .retained(true)
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
    ) -> (HomieNodeDescription, WaterSensorNodePublisher) {
        (
            self.node_builder.build(),
            WaterSensorNodePublisher::new(
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
pub struct WaterSensorNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    detected_prop: HomieID,
}

impl WaterSensorNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            detected_prop: WATER_SENSOR_NODE_DETECTED_PROP_ID.try_into().unwrap(),
        }
    }

    pub fn detected(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.detected_prop,
            value.to_string(),
            true,
        )
    }
}

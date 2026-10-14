use homie5::{
    Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};

use crate::{PropSpec, SMARTHOME_CAP_WATER_SENSOR};

pub const WATER_SENSOR_NODE_DEFAULT_ID: HomieID = HomieID::new_const("water");
pub const WATER_SENSOR_NODE_DEFAULT_NAME: &str = "Water sensor";

pub const WATER_SENSOR_NODE_DETECTED_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("detected"),
    retained: true,
    settable: false,
};

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
                .r#type(SMARTHOME_CAP_WATER_SENSOR);

        Self { node_builder: db }
    }
}

impl WaterSensorNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            WATER_SENSOR_NODE_DETECTED_PROP.id,
            PropertyDescriptionBuilder::boolean()
                .name("Water detection")
                .boolean_labels("no water", "water detected")
                .settable(WATER_SENSOR_NODE_DETECTED_PROP.settable)
                .retained(WATER_SENSOR_NODE_DETECTED_PROP.retained)
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
}

impl WaterSensorNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn detected(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &WATER_SENSOR_NODE_DETECTED_PROP.id,
            value.to_string(),
            WATER_SENSOR_NODE_DETECTED_PROP.retained,
        )
    }
}

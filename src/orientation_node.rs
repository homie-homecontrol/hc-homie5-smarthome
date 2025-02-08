use homie5::{
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
    Homie5DeviceProtocol, HomieID, NodeRef, HOMIE_UNIT_DEGREE,
};

use crate::SMARTHOME_TYPE_ORIENTATION;

pub const ORIENTATION_NODE_DEFAULT_ID: &str = "orientation";
pub const ORIENTATION_NODE_DEFAULT_NAME: &str = "Orientation sensor";
pub const ORIENTATION_NODE_ORIENT_X_PROP_ID: &str = "orientation-x";
pub const ORIENTATION_NODE_ORIENT_Y_PROP_ID: &str = "orientation-y";
pub const ORIENTATION_NODE_ORIENT_Z_PROP_ID: &str = "orientation-z";
pub const ORIENTATION_NODE_TILT_PROP_ID: &str = "tilt";

#[derive(Debug)]
pub struct OrientationNode {
    pub publisher: OrientationNodePublisher,
    pub orientation_x: i64,
    pub orientation_y: i64,
    pub orientation_z: i64,
    pub tilt: i64,
}

pub struct OrientationNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl Default for OrientationNodeBuilder {
    fn default() -> Self {
        let db =
            Self::build_node(NodeDescriptionBuilder::new().name(ORIENTATION_NODE_DEFAULT_NAME))
                .r#type(SMARTHOME_TYPE_ORIENTATION);

        Self { node_builder: db }
    }
}

impl OrientationNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            ORIENTATION_NODE_ORIENT_X_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Rotation X-Axis")
                .retained(true)
                .settable(false)
                .unit(HOMIE_UNIT_DEGREE)
                .build(),
        )
        .add_property(
            ORIENTATION_NODE_ORIENT_Y_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Rotation Y-Axist")
                .retained(true)
                .settable(false)
                .unit(HOMIE_UNIT_DEGREE)
                .build(),
        )
        .add_property(
            ORIENTATION_NODE_ORIENT_Z_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Rotation Z-Axist")
                .retained(true)
                .settable(false)
                .unit(HOMIE_UNIT_DEGREE)
                .build(),
        )
        .add_property(
            ORIENTATION_NODE_TILT_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Tilt angle")
                .retained(true)
                .settable(false)
                .unit(HOMIE_UNIT_DEGREE)
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
    ) -> (HomieNodeDescription, OrientationNodePublisher) {
        (
            self.node_builder.build(),
            OrientationNodePublisher::new(
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
pub struct OrientationNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    orient_x_prop: HomieID,
    orient_y_prop: HomieID,
    orient_z_prop: HomieID,
    tilt_prop: HomieID,
}

impl OrientationNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            orient_x_prop: ORIENTATION_NODE_ORIENT_X_PROP_ID.try_into().unwrap(),
            orient_y_prop: ORIENTATION_NODE_ORIENT_Y_PROP_ID.try_into().unwrap(),
            orient_z_prop: ORIENTATION_NODE_ORIENT_Z_PROP_ID.try_into().unwrap(),
            tilt_prop: ORIENTATION_NODE_TILT_PROP_ID.try_into().unwrap(),
        }
    }

    pub fn orientation_x(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.orient_x_prop,
            value.to_string(),
            true,
        )
    }

    pub fn orientation_y(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.orient_y_prop,
            value.to_string(),
            true,
        )
    }

    pub fn orientation_z(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.orient_z_prop,
            value.to_string(),
            true,
        )
    }

    pub fn tilt(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.tilt_prop,
            value.to_string(),
            true,
        )
    }
}

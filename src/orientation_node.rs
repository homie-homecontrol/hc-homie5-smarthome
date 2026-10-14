use homie5::{
    HOMIE_UNIT_DEGREE, Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};

use crate::{PropSpec, SMARTHOME_CAP_ORIENTATION};

pub const ORIENTATION_NODE_DEFAULT_ID: HomieID = HomieID::new_const("orientation");
pub const ORIENTATION_NODE_DEFAULT_NAME: &str = "Orientation sensor";

pub const ORIENTATION_NODE_ORIENT_X_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("orientation-x"),
    retained: true,
    settable: false,
};
pub const ORIENTATION_NODE_ORIENT_Y_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("orientation-y"),
    retained: true,
    settable: false,
};
pub const ORIENTATION_NODE_ORIENT_Z_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("orientation-z"),
    retained: true,
    settable: false,
};
pub const ORIENTATION_NODE_TILT_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("tilt"),
    retained: true,
    settable: false,
};

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
                .r#type(SMARTHOME_CAP_ORIENTATION);

        Self { node_builder: db }
    }
}

impl OrientationNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            ORIENTATION_NODE_ORIENT_X_PROP.id,
            PropertyDescriptionBuilder::integer()
                .name("Rotation X-Axis")
                .retained(ORIENTATION_NODE_ORIENT_X_PROP.retained)
                .settable(ORIENTATION_NODE_ORIENT_X_PROP.settable)
                .unit(HOMIE_UNIT_DEGREE)
                .build(),
        )
        .add_property(
            ORIENTATION_NODE_ORIENT_Y_PROP.id,
            PropertyDescriptionBuilder::integer()
                .name("Rotation Y-Axis")
                .retained(ORIENTATION_NODE_ORIENT_Y_PROP.retained)
                .settable(ORIENTATION_NODE_ORIENT_Y_PROP.settable)
                .unit(HOMIE_UNIT_DEGREE)
                .build(),
        )
        .add_property(
            ORIENTATION_NODE_ORIENT_Z_PROP.id,
            PropertyDescriptionBuilder::integer()
                .name("Rotation Z-Axis")
                .retained(ORIENTATION_NODE_ORIENT_Z_PROP.retained)
                .settable(ORIENTATION_NODE_ORIENT_Z_PROP.settable)
                .unit(HOMIE_UNIT_DEGREE)
                .build(),
        )
        .add_property(
            ORIENTATION_NODE_TILT_PROP.id,
            PropertyDescriptionBuilder::integer()
                .name("Tilt angle")
                .retained(ORIENTATION_NODE_TILT_PROP.retained)
                .settable(ORIENTATION_NODE_TILT_PROP.settable)
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
}

impl OrientationNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn orientation_x(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &ORIENTATION_NODE_ORIENT_X_PROP.id,
            value.to_string(),
            ORIENTATION_NODE_ORIENT_X_PROP.retained,
        )
    }

    pub fn orientation_y(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &ORIENTATION_NODE_ORIENT_Y_PROP.id,
            value.to_string(),
            ORIENTATION_NODE_ORIENT_Y_PROP.retained,
        )
    }

    pub fn orientation_z(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &ORIENTATION_NODE_ORIENT_Z_PROP.id,
            value.to_string(),
            ORIENTATION_NODE_ORIENT_Z_PROP.retained,
        )
    }

    pub fn tilt(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &ORIENTATION_NODE_TILT_PROP.id,
            value.to_string(),
            ORIENTATION_NODE_TILT_PROP.retained,
        )
    }
}

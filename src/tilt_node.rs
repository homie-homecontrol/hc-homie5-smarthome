use homie5::{
    Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};

use crate::{PropSpec, SMARTHOME_CAP_TILT};

pub const TILT_NODE_DEFAULT_ID: HomieID = HomieID::new_const("tilt");
pub const TILT_NODE_DEFAULT_NAME: &str = "Tilt sensor";

pub const TILT_NODE_STATE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("state"),
    retained: true,
    settable: false,
};

#[derive(Debug)]
pub struct TiltNode {
    pub publisher: TiltNodePublisher,
    pub state: bool,
}

pub struct TiltNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl Default for TiltNodeBuilder {
    fn default() -> Self {
        let db = Self::build_node(NodeDescriptionBuilder::new().name(TILT_NODE_DEFAULT_NAME))
            .r#type(SMARTHOME_CAP_TILT);

        Self { node_builder: db }
    }
}

impl TiltNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            TILT_NODE_STATE_PROP.id,
            PropertyDescriptionBuilder::boolean()
                .name("Tilted state")
                .boolean_labels("not tilted", "tilted")
                .settable(TILT_NODE_STATE_PROP.settable)
                .retained(TILT_NODE_STATE_PROP.retained)
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
    ) -> (HomieNodeDescription, TiltNodePublisher) {
        (
            self.node_builder.build(),
            TiltNodePublisher::new(
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
pub struct TiltNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
}

impl TiltNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn state(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &TILT_NODE_STATE_PROP.id,
            value.to_string(),
            TILT_NODE_STATE_PROP.retained,
        )
    }
}

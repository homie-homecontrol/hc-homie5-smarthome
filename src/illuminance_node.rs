use homie5::{
    HOMIE_UNIT_LUX, Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};

use crate::SMARTHOME_CAP_ILLUMINANCE;

pub const ILLUMINANCE_NODE_DEFAULT_ID: HomieID = HomieID::new_const("illuminance");
pub const ILLUMINANCE_NODE_DEFAULT_NAME: &str = "Illuminance sensor";
pub const ILLUMINANCE_NODE_ILLUMINANCE_PROP_ID: HomieID = HomieID::new_const("illuminance");

// ── Node (state) ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct IlluminanceNode {
    pub publisher: IlluminanceNodePublisher,
    pub illuminance: i64,
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct IlluminanceNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl Default for IlluminanceNodeBuilder {
    fn default() -> Self {
        let db =
            Self::build_node(NodeDescriptionBuilder::new().name(ILLUMINANCE_NODE_DEFAULT_NAME))
                .r#type(SMARTHOME_CAP_ILLUMINANCE);

        Self { node_builder: db }
    }
}

impl IlluminanceNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            ILLUMINANCE_NODE_ILLUMINANCE_PROP_ID,
            PropertyDescriptionBuilder::integer()
                .name("Illuminance")
                .unit(HOMIE_UNIT_LUX)
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
    ) -> (HomieNodeDescription, IlluminanceNodePublisher) {
        (
            self.node_builder.build(),
            IlluminanceNodePublisher::new(
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

// ── Publisher ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct IlluminanceNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    illuminance_prop: HomieID,
}

impl IlluminanceNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            illuminance_prop: ILLUMINANCE_NODE_ILLUMINANCE_PROP_ID,
        }
    }

    pub fn illuminance(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.illuminance_prop,
            value.to_string(),
            true,
        )
    }
}

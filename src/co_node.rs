use homie5::{
    Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};

use crate::SMARTHOME_CAP_CO;

pub const CO_NODE_DEFAULT_ID: HomieID = HomieID::new_const("co");
pub const CO_NODE_DEFAULT_NAME: &str = "Carbon monoxide detector";
pub const CO_NODE_DETECTED_PROP_ID: HomieID = HomieID::new_const("detected");

// ── Node (state) ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct CoNode {
    pub publisher: CoNodePublisher,
    pub detected: bool,
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct CoNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl Default for CoNodeBuilder {
    fn default() -> Self {
        let db = Self::build_node(NodeDescriptionBuilder::new().name(CO_NODE_DEFAULT_NAME))
            .r#type(SMARTHOME_CAP_CO);

        Self { node_builder: db }
    }
}

impl CoNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            CO_NODE_DETECTED_PROP_ID,
            PropertyDescriptionBuilder::boolean()
                .name("CO detected")
                .boolean_labels("clear", "co detected")
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
    ) -> (HomieNodeDescription, CoNodePublisher) {
        (
            self.node_builder.build(),
            CoNodePublisher::new(
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
pub struct CoNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    detected_prop: HomieID,
}

impl CoNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            detected_prop: CO_NODE_DETECTED_PROP_ID,
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

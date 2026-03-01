use homie5::{
    Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        BooleanFormat, HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};

use crate::SMARTHOME_CAP_SMOKE;

pub const SMOKE_NODE_DEFAULT_ID: HomieID = HomieID::new_const("smoke");
pub const SMOKE_NODE_DEFAULT_NAME: &str = "Smoke detector";
pub const SMOKE_NODE_DETECTED_PROP_ID: HomieID = HomieID::new_const("detected");

// ── Node (state) ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct SmokeNode {
    pub publisher: SmokeNodePublisher,
    pub detected: bool,
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct SmokeNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl Default for SmokeNodeBuilder {
    fn default() -> Self {
        let db = Self::build_node(NodeDescriptionBuilder::new().name(SMOKE_NODE_DEFAULT_NAME))
            .r#type(SMARTHOME_CAP_SMOKE);

        Self { node_builder: db }
    }
}

impl SmokeNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            SMOKE_NODE_DETECTED_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                .name("Smoke detected")
                .format(HomiePropertyFormat::Boolean(BooleanFormat {
                    false_val: "no smoke".to_owned(),
                    true_val: "smoke detected".to_owned(),
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
    ) -> (HomieNodeDescription, SmokeNodePublisher) {
        (
            self.node_builder.build(),
            SmokeNodePublisher::new(
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
pub struct SmokeNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    detected_prop: HomieID,
}

impl SmokeNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            detected_prop: SMOKE_NODE_DETECTED_PROP_ID,
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

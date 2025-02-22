use homie5::{
    device_description::{
        BooleanFormat, HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
    Homie5DeviceProtocol, HomieID, NodeRef,
};

use crate::SMARTHOME_TYPE_TILT;

pub const TILT_NODE_DEFAULT_ID: &str = "tilt";
pub const TILT_NODE_DEFAULT_NAME: &str = "Tilt sensor";
pub const TILT_NODE_STATE_PROP_ID: &str = "state";

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
            .r#type(SMARTHOME_TYPE_TILT);

        Self { node_builder: db }
    }
}

impl TiltNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            TILT_NODE_STATE_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                .name("Tilted state")
                .format(HomiePropertyFormat::Boolean(BooleanFormat {
                    false_val: "not tilted".to_owned(),
                    true_val: "tilted".to_owned(),
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
    state_prop: HomieID,
}

impl TiltNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            state_prop: TILT_NODE_STATE_PROP_ID.try_into().unwrap(),
        }
    }

    pub fn state(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.state_prop,
            value.to_string(),
            true,
        )
    }
}

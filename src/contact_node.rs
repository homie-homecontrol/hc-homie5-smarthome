use homie5::{
    device_description::{
        BooleanFormat, HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
    Homie5DeviceProtocol, HomieID, NodeRef,
};

use crate::SMARTHOME_TYPE_CONTACT;

pub const CONTACT_NODE_DEFAULT_ID: &str = "contact";
pub const CONTACT_NODE_DEFAULT_NAME: &str = "Open/Close contact";
pub const CONTACT_NODE_STATE_PROP_ID: HomieID = HomieID::new_const("state");

#[derive(Debug)]
pub struct ContactNode {
    pub publisher: ContactNodePublisher,
    pub state: bool,
}

pub struct ContactNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl Default for ContactNodeBuilder {
    fn default() -> Self {
        let db = Self::build_node(NodeDescriptionBuilder::new().name(CONTACT_NODE_DEFAULT_NAME))
            .r#type(SMARTHOME_TYPE_CONTACT);

        Self { node_builder: db }
    }
}

impl ContactNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            CONTACT_NODE_STATE_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                .name("Open/Close state")
                .format(HomiePropertyFormat::Boolean(BooleanFormat {
                    false_val: "closed".to_owned(),
                    true_val: "open".to_owned(),
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
    ) -> (HomieNodeDescription, ContactNodePublisher) {
        (
            self.node_builder.build(),
            ContactNodePublisher::new(
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
pub struct ContactNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    state_prop: HomieID,
}

impl ContactNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            state_prop: CONTACT_NODE_STATE_PROP_ID,
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

use homie5::{
    HOMIE_UNIT_DEGREE, Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};

use crate::{PropSpec, SMARTHOME_CAP_ROTARY};

pub const ROTARY_NODE_DEFAULT_ID: HomieID = HomieID::new_const("rotary");
pub const ROTARY_NODE_DEFAULT_NAME: &str = "Rotary control";

pub const ROTARY_NODE_ROTATION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("rotation"),
    retained: false,
    settable: false,
};

// ── Builder ─────────────────────────────────────────────────────────────────

// A relative rotary input (dial, knob, scroll wheel, or a whole device being turned) only
// publishes events, so there is no `RotaryNode` state struct: callers keep the publisher.

pub struct RotaryNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl Default for RotaryNodeBuilder {
    fn default() -> Self {
        let db = Self::build_node(NodeDescriptionBuilder::new().name(ROTARY_NODE_DEFAULT_NAME))
            .r#type(SMARTHOME_CAP_ROTARY);

        Self { node_builder: db }
    }
}

impl RotaryNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            ROTARY_NODE_ROTATION_PROP.id,
            PropertyDescriptionBuilder::float()
                .name("Rotation")
                .unit(HOMIE_UNIT_DEGREE)
                .settable(ROTARY_NODE_ROTATION_PROP.settable)
                .retained(ROTARY_NODE_ROTATION_PROP.retained)
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
    ) -> (HomieNodeDescription, RotaryNodePublisher) {
        (
            self.node_builder.build(),
            RotaryNodePublisher::new(
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
pub struct RotaryNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
}

impl RotaryNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    /// Publishes a turn of `degrees`: positive is clockwise, negative is counter-clockwise.
    pub fn rotation(&self, degrees: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &ROTARY_NODE_ROTATION_PROP.id,
            degrees.to_string(),
            ROTARY_NODE_ROTATION_PROP.retained,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use homie5::{HomieDataType, HomieDomain, client::QoS};

    #[test]
    fn rotation_is_a_non_retained_float_in_degrees() {
        let node = RotaryNodeBuilder::new().build();
        let rotation = node.properties.get(&ROTARY_NODE_ROTATION_PROP.id).unwrap();

        assert_eq!(node.r#type.as_deref(), Some(SMARTHOME_CAP_ROTARY));
        assert_eq!(rotation.datatype, HomieDataType::Float);
        assert_eq!(rotation.unit.as_deref(), Some(HOMIE_UNIT_DEGREE));
        assert!(!rotation.retained);
        assert!(!rotation.settable);
        assert_eq!(node.properties.len(), 1);
    }

    #[test]
    fn rotation_publishes_signed_degrees() {
        let (client, _) =
            Homie5DeviceProtocol::new(HomieID::new_const("dev"), HomieDomain::Default);
        let (_, publisher) =
            RotaryNodeBuilder::new().build_with_publisher(ROTARY_NODE_DEFAULT_ID, &client);

        let clockwise = publisher.rotation(22.5);
        assert!(clockwise.topic.ends_with("/rotary/rotation"));
        assert_eq!(clockwise.payload, b"22.5");
        assert!(!clockwise.retain);
        assert_eq!(clockwise.qos, QoS::AtMostOnce);

        assert_eq!(publisher.rotation(-40.0).payload, b"-40");
    }
}

use homie5::{
    Homie5DeviceProtocol, Homie5Message, HomieID, HomieValue, NodeRef, PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_CAP_VALVE, SetCommandParser};

pub const VALVE_NODE_DEFAULT_ID: HomieID = HomieID::new_const("valve");
pub const VALVE_NODE_DEFAULT_NAME: &str = "Valve control";
pub const VALVE_NODE_STATE_PROP_ID: HomieID = HomieID::new_const("state");

#[derive(Debug)]
pub struct ValveNode {
    pub publisher: ValveNodePublisher,
    pub state: bool,
    pub state_target: bool,
}

#[derive(Debug)]
pub enum ValveNodeSetEvents {
    State(bool),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ValveNodeConfig {
    pub settable: bool,
}

impl Default for ValveNodeConfig {
    fn default() -> Self {
        Self { settable: true }
    }
}

pub struct ValveNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl ValveNodeBuilder {
    pub fn new(config: &ValveNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(VALVE_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_VALVE);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &ValveNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            VALVE_NODE_STATE_PROP_ID,
            PropertyDescriptionBuilder::boolean()
                .name("Valve state")
                .boolean_labels("closed", "open")
                .settable(config.settable)
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
    ) -> (HomieNodeDescription, ValveNodePublisher) {
        (
            self.node_builder.build(),
            ValveNodePublisher::new(
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
pub struct ValveNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    state_prop: HomieID,
}

impl ValveNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            state_prop: VALVE_NODE_STATE_PROP_ID,
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

    pub fn state_target(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &self.state_prop,
            value.to_string(),
            true,
        )
    }
}

impl SetCommandParser for ValveNodePublisher {
    type Event = ValveNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &self.state_prop) {
            let Some(parsed) = desc.with_property(property, |prop_desc| {
                HomieValue::parse(set_value, prop_desc)
            }) else {
                return ParseOutcome::Invalid(ParseError::new(
                    property_id,
                    set_value,
                    ParseErrorKind::MissingPropertyDescription,
                ));
            };

            match parsed {
                Ok(HomieValue::Bool(value)) => {
                    ParseOutcome::Parsed(ValveNodeSetEvents::State(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else {
            ParseOutcome::NoMatch
        }
    }

    fn parse_set_event(
        &self,
        desc: &HomieDeviceDescription,
        event: &Homie5Message,
    ) -> ParseOutcome<Self::Event> {
        match event {
            Homie5Message::PropertySet {
                property,
                set_value,
            } => self.parse_set(property, desc, set_value),
            _ => ParseOutcome::Invalid(ParseError::new(
                self.state_prop.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

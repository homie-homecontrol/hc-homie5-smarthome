use std::{fmt::Display, str::FromStr};

use homie5::{
    HOMIE_UNIT_PERCENT, Homie5DeviceProtocol, Homie5Message, Homie5ProtocolError, HomieID,
    HomieValue, NodeRef, PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat, IntegerRange,
        NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_TYPE_SHUTTER;

pub const SHUTTER_NODE_DEFAULT_ID: &str = "shutter";
pub const SHUTTER_NODE_DEFAULT_NAME: &str = "Shutter control";
pub const SHUTTER_NODE_POSITION_PROP_ID: &str = "position";
pub const SHUTTER_NODE_ACTION_PROP_ID: &str = "action";

#[derive(Debug)]
pub struct ShutterNode {
    pub publisher: ShutterNodePublisher,
    pub position: i64,
    pub position_target: i64,
}

#[derive(Debug)]
pub enum ShutterNodeActions {
    Up,
    Down,
    Stop,
}

impl Display for ShutterNodeActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &'static str = self.into();
        write!(f, "{}", s)
    }
}

impl From<&ShutterNodeActions> for &'static str {
    fn from(action: &ShutterNodeActions) -> Self {
        match action {
            ShutterNodeActions::Up => "up",
            ShutterNodeActions::Down => "down",
            ShutterNodeActions::Stop => "stop",
        }
    }
}
impl FromStr for ShutterNodeActions {
    type Err = Homie5ProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "up" => Ok(ShutterNodeActions::Up),
            "down" => Ok(ShutterNodeActions::Down),
            "stop" => Ok(ShutterNodeActions::Stop),
            _ => Err(Homie5ProtocolError::InvalidPayload),
        }
    }
}

#[derive(Debug)]
pub enum ShutterNodeSetEvents {
    Position(i64),
    Action(ShutterNodeActions),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ShutterNodeConfig {
    pub can_stop: bool,
}

impl Default for ShutterNodeConfig {
    fn default() -> Self {
        Self { can_stop: true }
    }
}

pub struct ShutterNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl ShutterNodeBuilder {
    pub fn new(config: &ShutterNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(SHUTTER_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_TYPE_SHUTTER);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &ShutterNodeConfig,
    ) -> NodeDescriptionBuilder {
        let mut actions = vec![ShutterNodeActions::Up, ShutterNodeActions::Down];

        if config.can_stop {
            actions.push(ShutterNodeActions::Stop);
        }

        db.add_property(
            SHUTTER_NODE_POSITION_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Shutter position")
                .format(HomiePropertyFormat::IntegerRange(IntegerRange {
                    min: Some(0),
                    max: Some(100),
                    step: None,
                }))
                .unit(HOMIE_UNIT_PERCENT)
                .settable(true)
                .retained(true)
                .build(),
        )
        .add_property(
            SHUTTER_NODE_ACTION_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Control Shutter")
                .format(HomiePropertyFormat::Enum(
                    actions.iter().map(|a| a.to_string()).collect(),
                ))
                .settable(true)
                .retained(false)
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
    ) -> (HomieNodeDescription, ShutterNodePublisher) {
        let did = client.id().clone();
        (
            self.node_builder.build(),
            ShutterNodePublisher::new(
                NodeRef::new(client.homie_domain().to_owned(), did, node_id),
                client.clone(),
            ),
        )
    }
}

#[derive(Debug)]
pub struct ShutterNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    position_prop: HomieID,
    action_prop: HomieID,
}

impl ShutterNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            position_prop: SHUTTER_NODE_POSITION_PROP_ID.try_into().unwrap(),
            action_prop: SHUTTER_NODE_ACTION_PROP_ID.try_into().unwrap(),
        }
    }

    pub fn position(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.position_prop,
            value.to_string(),
            true,
        )
    }

    pub fn position_target(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &self.position_prop,
            value.to_string(),
            true,
        )
    }

    pub fn action(&self, action: ShutterNodeActions) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.action_prop,
            action.to_string(),
            false,
        )
    }

    pub fn match_parse(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> Option<ShutterNodeSetEvents> {
        if property.match_with_node(&self.node, &self.position_prop) {
            desc.with_property(property, |prop_desc| {
                if let Ok(HomieValue::Integer(value)) = HomieValue::parse(set_value, prop_desc) {
                    Some(ShutterNodeSetEvents::Position(value))
                } else {
                    None
                }
            })?
        } else if property.match_with_node(&self.node, &self.action_prop) {
            desc.with_property(property, |prop_desc| {
                if let Ok(HomieValue::Enum(value)) = HomieValue::parse(set_value, prop_desc) {
                    if let Ok(value) = ShutterNodeActions::from_str(&value) {
                        Some(ShutterNodeSetEvents::Action(value))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })?
        } else {
            None
        }
    }

    pub fn match_parse_event(
        &self,
        desc: &HomieDeviceDescription,
        event: &Homie5Message,
    ) -> Option<ShutterNodeSetEvents> {
        match event {
            Homie5Message::PropertySet {
                property,
                set_value,
            } => self.match_parse(property, desc, set_value),
            _ => None,
        }
    }
}

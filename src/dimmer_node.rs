use std::str::FromStr;

use homie5::{
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat, IntegerRange,
        NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
    Homie5DeviceProtocol, Homie5Message, Homie5ProtocolError, HomieID, HomieValue, NodeRef,
    PropertyRef, HOMIE_UNIT_PERCENT,
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_TYPE_DIMMER;

pub const DIMMER_NODE_DEFAULT_ID: &str = "dimmer";
pub const DIMMER_NODE_DEFAULT_NAME: &str = "Brightness control";
pub const DIMMER_NODE_BRIGHTNESS_PROP_ID: &str = "brightness";
pub const DIMMER_NODE_ACTION_PROP_ID: &str = "action";

#[derive(Debug)]
pub struct DimmerNode {
    pub publisher: DimmerNodePublisher,
    pub state: i64,
    pub state_target: i64,
}

#[derive(Debug)]
pub enum DimmerNodeActions {
    Brighter,
    Darker,
}

impl FromStr for DimmerNodeActions {
    type Err = Homie5ProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "brighter" => Ok(DimmerNodeActions::Brighter),
            "darker" => Ok(DimmerNodeActions::Darker),
            _ => Err(Homie5ProtocolError::InvalidPayload),
        }
    }
}

#[derive(Debug)]
pub enum DimmerNodeSetEvents {
    Brightness(i64),
    Action(DimmerNodeActions),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DimmerNodeConfig {
    pub settable: bool,
}

impl Default for DimmerNodeConfig {
    fn default() -> Self {
        Self { settable: true }
    }
}

pub struct DimmerNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl DimmerNodeBuilder {
    pub fn new(config: &DimmerNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(DIMMER_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_TYPE_DIMMER);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &DimmerNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            DIMMER_NODE_BRIGHTNESS_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Brightness Level")
                .format(HomiePropertyFormat::IntegerRange(IntegerRange {
                    min: Some(0),
                    max: Some(100),
                    step: None,
                }))
                .unit(HOMIE_UNIT_PERCENT)
                .settable(config.settable)
                .retained(true)
                .build(),
        )
        .add_property(
            DIMMER_NODE_ACTION_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Change Brightness")
                .format(HomiePropertyFormat::Enum(vec![
                    "brighter".to_owned(),
                    "darker".to_owned(),
                ]))
                .settable(config.settable)
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
    ) -> (HomieNodeDescription, DimmerNodePublisher) {
        let did = client.id().clone();
        (
            self.node_builder.build(),
            DimmerNodePublisher::new(
                NodeRef::new(client.homie_domain().to_owned(), did, node_id),
                client.clone(),
            ),
        )
    }
}

#[derive(Debug)]
pub struct DimmerNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    brightness_prop: HomieID,
    action_prop: HomieID,
}

impl DimmerNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            brightness_prop: DIMMER_NODE_BRIGHTNESS_PROP_ID.try_into().unwrap(),
            action_prop: DIMMER_NODE_ACTION_PROP_ID.try_into().unwrap(),
        }
    }

    pub fn brightness(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.brightness_prop,
            value.to_string(),
            true,
        )
    }

    pub fn brightness_target(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &self.brightness_prop,
            value.to_string(),
            true,
        )
    }

    pub fn action(&self, action: DimmerNodeActions) -> homie5::client::Publish {
        let action_str = match action {
            DimmerNodeActions::Brighter => "brighter",
            DimmerNodeActions::Darker => "darker",
        };
        self.client
            .publish_value(self.node.node_id(), &self.action_prop, action_str, false)
    }

    pub fn match_parse(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> Option<DimmerNodeSetEvents> {
        if property.match_with_node(&self.node, &self.brightness_prop) {
            desc.with_property(property, |prop_desc| {
                if let Ok(HomieValue::Integer(value)) = HomieValue::parse(set_value, prop_desc) {
                    Some(DimmerNodeSetEvents::Brightness(value))
                } else {
                    None
                }
            })?
        } else if property.match_with_node(&self.node, &self.action_prop) {
            desc.with_property(property, |prop_desc| {
                if let Ok(HomieValue::Enum(value)) = HomieValue::parse(set_value, prop_desc) {
                    if let Ok(value) = DimmerNodeActions::from_str(&value) {
                        Some(DimmerNodeSetEvents::Action(value))
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
    ) -> Option<DimmerNodeSetEvents> {
        match event {
            Homie5Message::PropertySet {
                property,
                set_value,
            } => self.match_parse(property, desc, set_value),
            _ => None,
        }
    }
}

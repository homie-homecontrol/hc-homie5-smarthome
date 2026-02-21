use std::{fmt::Display, str::FromStr};

use homie5::{
    Homie5DeviceProtocol, Homie5ProtocolError, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_TYPE_BUTTON;

pub const BUTTON_NODE_DEFAULT_ID: &str = "button";
pub const BUTTON_NODE_DEFAULT_NAME: &str = "Pushbutton";

pub const BUTTON_NODE_ACTION_PROP_ID: &str = "action";

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ButtonNodeActions {
    Press,
    LongPress,
    DoublePress,
    Release,
    LongRelease,
    Continuous,
}

impl FromStr for ButtonNodeActions {
    type Err = Homie5ProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "press" => Ok(ButtonNodeActions::Press),
            _ => Err(Homie5ProtocolError::InvalidPayload),
        }
    }
}

impl Display for ButtonNodeActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &'static str = self.into();
        write!(f, "{}", s)
    }
}

impl From<&ButtonNodeActions> for &'static str {
    fn from(action: &ButtonNodeActions) -> Self {
        match action {
            ButtonNodeActions::Press => "press",
            ButtonNodeActions::LongPress => "long-press",
            ButtonNodeActions::DoublePress => "double-press",
            ButtonNodeActions::Release => "release",
            ButtonNodeActions::LongRelease => "long-release",
            ButtonNodeActions::Continuous => "continuous",
        }
    }
}

impl ButtonNodeActions {
    pub fn all_variants() -> &'static [Self] {
        &[ButtonNodeActions::Press]
    }

    pub fn to_string_vec() -> Vec<String> {
        Self::all_variants().iter().map(|v| v.to_string()).collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ButtonNodeConfig {
    pub actions: Vec<ButtonNodeActions>,
}

impl Default for ButtonNodeConfig {
    fn default() -> Self {
        Self {
            actions: vec![ButtonNodeActions::Press],
        }
    }
}

pub struct ButtonNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl ButtonNodeBuilder {
    pub fn new(config: &ButtonNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(BUTTON_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_TYPE_BUTTON);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &ButtonNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            BUTTON_NODE_ACTION_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Button action event")
                .format(HomiePropertyFormat::Enum(
                    config.actions.iter().map(|a| a.to_string()).collect(),
                ))
                .settable(false)
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
    ) -> (HomieNodeDescription, ButtonNodePublisher) {
        (
            self.node_builder.build(),
            ButtonNodePublisher::new(
                NodeRef::new(client.homie_domain().clone(), client.id().clone(), node_id),
                client.clone(),
            ),
        )
    }
}

impl Default for ButtonNodeBuilder {
    fn default() -> Self {
        Self::new(&ButtonNodeConfig::default())
    }
}

#[derive(Debug)]
pub struct ButtonNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    action_prop: HomieID,
}

impl ButtonNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            action_prop: BUTTON_NODE_ACTION_PROP_ID.try_into().unwrap(),
        }
    }

    pub fn action(&self, kind: &ButtonNodeActions) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.action_prop,
            kind.to_string(),
            false,
        )
    }
}

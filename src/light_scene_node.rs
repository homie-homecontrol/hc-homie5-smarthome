use homie5::{
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
    Homie5DeviceProtocol, Homie5Message, HomieID, HomieValue, NodeRef, PropertyRef,
};
use serde::{Deserialize, Serialize};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, SetCommandParser, SMARTHOME_TYPE_LIGHTSCENE,
};

pub const LIGHTSCENE_NODE_DEFAULT_ID: &str = "scenes";
pub const LIGHTSCENE_NODE_DEFAULT_NAME: &str = "Light scenes";
pub const LIGHTSCENE_NODE_RECALL_PROP_ID: &str = "recall";

#[derive(Debug)]
pub enum LightSceneNodeActions {
    Recall(String),
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LightSceneNodeConfig {
    pub scenes: Vec<String>,
    pub settable: bool,
}

pub struct LightSceneNodeBuilder {
    node_builder: NodeDescriptionBuilder,
    config: LightSceneNodeConfig,
}

impl LightSceneNodeBuilder {
    pub fn new(config: &LightSceneNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(LIGHTSCENE_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_TYPE_LIGHTSCENE);

        Self {
            node_builder: db,
            config: config.clone(),
        }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &LightSceneNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            LIGHTSCENE_NODE_RECALL_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Enum)
                .name("Recall a scene")
                .format(HomiePropertyFormat::Enum(config.scenes.clone()))
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
    ) -> (HomieNodeDescription, LightSceneNodePublisher) {
        (
            self.node_builder.build(),
            LightSceneNodePublisher::new(
                NodeRef::new(
                    client.homie_domain().to_owned(),
                    client.id().to_owned(),
                    node_id,
                ),
                self.config,
                client.clone(),
            ),
        )
    }
}

#[derive(Debug)]
pub struct LightSceneNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    recall_prop: HomieID,
    config: LightSceneNodeConfig,
}

impl LightSceneNodePublisher {
    pub fn new(node: NodeRef, config: LightSceneNodeConfig, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            config,
            client,
            recall_prop: LIGHTSCENE_NODE_RECALL_PROP_ID.try_into().unwrap(),
        }
    }

    pub fn recall(
        &self,
        LightSceneNodeActions::Recall(scene): &LightSceneNodeActions,
    ) -> Option<homie5::client::Publish> {
        if self.config.scenes.contains(scene) {
            Some(
                self.client
                    .publish_value(self.node.node_id(), &self.recall_prop, scene, false),
            )
        } else {
            None
        }
    }
}

impl SetCommandParser for LightSceneNodePublisher {
    type Event = LightSceneNodeActions;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        if property.match_with_node(&self.node, &self.recall_prop) {
            let Some(parsed) = desc.with_property(property, |prop_desc| {
                HomieValue::parse(set_value, prop_desc)
            }) else {
                return ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::MissingPropertyDescription,
                ));
            };

            match parsed {
                Ok(HomieValue::Enum(value)) => {
                    ParseOutcome::Parsed(LightSceneNodeActions::Recall(value))
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
                self.recall_prop.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

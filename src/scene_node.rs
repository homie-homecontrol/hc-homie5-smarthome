use homie5::{
    Homie5DeviceProtocol, Homie5Message, HomieID, HomieValue, NodeRef, PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_CAP_SCENE, SetCommandParser};

pub const SCENE_NODE_DEFAULT_ID: HomieID = HomieID::new_const("scene");
pub const SCENE_NODE_DEFAULT_NAME: &str = "Scene recall";
pub const SCENE_NODE_RECALL_PROP_ID: HomieID = HomieID::new_const("recall");

#[derive(Debug)]
pub enum SceneNodeActions {
    Recall(String),
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SceneNodeConfig {
    pub scenes: Vec<String>,
    pub settable: bool,
}

pub struct SceneNodeBuilder {
    node_builder: NodeDescriptionBuilder,
    config: SceneNodeConfig,
}

impl SceneNodeBuilder {
    pub fn new(config: &SceneNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(SCENE_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_SCENE);

        Self {
            node_builder: db,
            config: config.clone(),
        }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &SceneNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            SCENE_NODE_RECALL_PROP_ID,
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
    ) -> (HomieNodeDescription, SceneNodePublisher) {
        (
            self.node_builder.build(),
            SceneNodePublisher::new(
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
pub struct SceneNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    recall_prop: HomieID,
    config: SceneNodeConfig,
}

impl SceneNodePublisher {
    pub fn new(node: NodeRef, config: SceneNodeConfig, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            config,
            client,
            recall_prop: SCENE_NODE_RECALL_PROP_ID,
        }
    }

    pub fn recall(
        &self,
        SceneNodeActions::Recall(scene): &SceneNodeActions,
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

impl SetCommandParser for SceneNodePublisher {
    type Event = SceneNodeActions;

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
                    ParseOutcome::Parsed(SceneNodeActions::Recall(value))
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

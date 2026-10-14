use homie5::{
    Homie5DeviceProtocol, Homie5Message, HomieID, HomieValue, NodeRef, PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, PropSpec, SMARTHOME_CAP_TEXT, SetCommandParser,
};

pub const TEXT_NODE_DEFAULT_ID: HomieID = HomieID::new_const("text");
pub const TEXT_NODE_DEFAULT_NAME: &str = "Text";

pub const TEXT_NODE_TEXT_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("text"),
    retained: true,
    settable: true,
};

// ── Node (state) ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct TextNode {
    pub publisher: TextNodePublisher,
    pub text: String,
}

// ── Set events ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum TextNodeSetEvents {
    Text(String),
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct TextNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl Default for TextNodeBuilder {
    fn default() -> Self {
        let db = Self::build_node(NodeDescriptionBuilder::new().name(TEXT_NODE_DEFAULT_NAME))
            .r#type(SMARTHOME_CAP_TEXT);

        Self { node_builder: db }
    }
}

impl TextNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn build_node(db: NodeDescriptionBuilder) -> NodeDescriptionBuilder {
        db.add_property(
            TEXT_NODE_TEXT_PROP.id,
            PropertyDescriptionBuilder::string()
                .name("Text content")
                .settable(TEXT_NODE_TEXT_PROP.settable)
                .retained(TEXT_NODE_TEXT_PROP.retained)
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
    ) -> (HomieNodeDescription, TextNodePublisher) {
        (
            self.node_builder.build(),
            TextNodePublisher::new(
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
pub struct TextNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
}

impl TextNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn text(&self, value: &str) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &TEXT_NODE_TEXT_PROP.id,
            value,
            TEXT_NODE_TEXT_PROP.retained,
        )
    }
}

impl SetCommandParser for TextNodePublisher {
    type Event = TextNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &TEXT_NODE_TEXT_PROP.id) {
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
                Ok(HomieValue::String(value)) => {
                    ParseOutcome::Parsed(TextNodeSetEvents::Text(value))
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
                TEXT_NODE_TEXT_PROP.id.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

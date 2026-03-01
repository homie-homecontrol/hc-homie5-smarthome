use homie5::{
    Homie5DeviceProtocol, Homie5Message, HomieColorValue, HomieID, HomieValue, NodeRef,
    PropertyRef,
    device_description::{
        ColorFormat, HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat,
        IntegerRange, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_CAP_COLOR, SetCommandParser};

pub const COLOR_NODE_DEFAULT_ID: HomieID = HomieID::new_const("color");
pub const COLOR_NODE_DEFAULT_NAME: &str = "Color control";
pub const COLOR_NODE_COLOR_PROP_ID: HomieID = HomieID::new_const("color");
pub const COLOR_NODE_COLOR_TEMP_PROP_ID: HomieID = HomieID::new_const("color-temperature");

#[derive(Debug)]
pub struct ColorNode {
    pub publisher: ColorNodePublisher,
    pub color: HomieColorValue,
    pub color_target: HomieColorValue,
    pub color_temperature: i64,
    pub color_temperature_target: i64,
}

#[derive(Debug)]
pub enum ColorNodeSetEvents {
    Color(HomieColorValue),
    ColorTemperature(i64),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorNodeConfig {
    pub settable: bool,
    pub color_formats: Vec<ColorFormat>,
    pub ctmin: i64,
    pub ctmax: i64,
}

impl Default for ColorNodeConfig {
    fn default() -> Self {
        Self {
            settable: true,
            color_formats: vec![ColorFormat::Rgb],
            ctmin: 153,
            ctmax: 555,
        }
    }
}

pub struct ColorNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl ColorNodeBuilder {
    pub fn new(config: &ColorNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(COLOR_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_COLOR);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &ColorNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            COLOR_NODE_COLOR_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Color)
                .name("Color")
                .format(HomiePropertyFormat::Color(config.color_formats.clone()))
                .settable(config.settable)
                .retained(true)
                .build(),
        )
        .add_property(
            COLOR_NODE_COLOR_TEMP_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                .name("Color temperature")
                .format(HomiePropertyFormat::IntegerRange(IntegerRange {
                    min: Some(config.ctmin),
                    max: Some(config.ctmax),
                    step: None,
                }))
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
    ) -> (HomieNodeDescription, ColorNodePublisher) {
        (
            self.node_builder.build(),
            ColorNodePublisher::new(
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
pub struct ColorNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    color_prop_id: HomieID,
    color_temp_prop_id: HomieID,
}

impl ColorNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            color_prop_id: COLOR_NODE_COLOR_PROP_ID,
            color_temp_prop_id: COLOR_NODE_COLOR_TEMP_PROP_ID,
        }
    }

    pub fn color(&self, value: HomieColorValue) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.color_prop_id, value, true)
    }

    pub fn color_target(&self, value: HomieColorValue) -> homie5::client::Publish {
        self.client
            .publish_target(self.node.node_id(), &self.color_prop_id, value, true)
    }

    pub fn color_temperature(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.color_temp_prop_id,
            value.to_string(),
            true,
        )
    }

    pub fn color_temperature_target(&self, value: i64) -> homie5::client::Publish {
        self.client.publish_target(
            self.node.node_id(),
            &self.color_temp_prop_id,
            value.to_string(),
            true,
        )
    }
}

impl SetCommandParser for ColorNodePublisher {
    type Event = ColorNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &self.color_prop_id) {
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
                Ok(HomieValue::Color(value)) => {
                    ParseOutcome::Parsed(ColorNodeSetEvents::Color(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &self.color_temp_prop_id) {
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
                Ok(HomieValue::Integer(value)) => {
                    ParseOutcome::Parsed(ColorNodeSetEvents::ColorTemperature(value))
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
                self.color_prop_id.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

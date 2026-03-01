use homie5::{
    device_description::{
        ColorFormat, HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat,
        IntegerRange, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
    Homie5DeviceProtocol, Homie5Message, HomieColorValue, HomieID, HomieValue, NodeRef,
    PropertyRef,
};
use serde::{Deserialize, Serialize};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, SetCommandParser, SMARTHOME_TYPE_COLORLIGHT,
};

pub const COLORLIGHT_NODE_DEFAULT_ID: &str = "colorlight";
pub const COLORLIGHT_NODE_DEFAULT_NAME: &str = "Colorlight control";
pub const COLORLIGHT_NODE_COLOR_PROP_ID: &str = "color";
pub const COLORLIGHT_NODE_COLOR_TEMP_PROP_ID: &str = "color-temperature";

#[derive(Debug)]
pub struct ColorlightNode {
    pub publisher: ColorlightNodePublisher,
    pub color: HomieColorValue,
    pub color_target: HomieColorValue,
    pub color_temperature: i64,
    pub color_temperature_target: i64,
}

#[derive(Debug)]
pub enum ColorlightNodeSetEvents {
    Color(HomieColorValue),
    ColorTemperature(i64),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ColorlightNodeConfig {
    pub settable: bool,
    pub color_formats: Vec<ColorFormat>,
    pub ctmin: i64,
    pub ctmax: i64,
}

impl Default for ColorlightNodeConfig {
    fn default() -> Self {
        Self {
            settable: true,
            color_formats: vec![ColorFormat::Rgb],
            ctmin: 153,
            ctmax: 555,
        }
    }
}

pub struct ColorlightNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl ColorlightNodeBuilder {
    pub fn new(config: &ColorlightNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(COLORLIGHT_NODE_DEFAULT_ID),
            config,
        )
        .r#type(SMARTHOME_TYPE_COLORLIGHT);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &ColorlightNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            COLORLIGHT_NODE_COLOR_PROP_ID.try_into().unwrap(),
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Color)
                .name("Color")
                .format(HomiePropertyFormat::Color(config.color_formats.clone()))
                .settable(config.settable)
                .retained(true)
                .build(),
        )
        .add_property(
            COLORLIGHT_NODE_COLOR_TEMP_PROP_ID.try_into().unwrap(),
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
    ) -> (HomieNodeDescription, ColorlightNodePublisher) {
        (
            self.node_builder.build(),
            ColorlightNodePublisher::new(
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
pub struct ColorlightNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    color_prop_id: HomieID,
    color_temp_prop_id: HomieID,
}

impl ColorlightNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            color_prop_id: COLORLIGHT_NODE_COLOR_PROP_ID.try_into().unwrap(),
            color_temp_prop_id: COLORLIGHT_NODE_COLOR_TEMP_PROP_ID.try_into().unwrap(),
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

impl SetCommandParser for ColorlightNodePublisher {
    type Event = ColorlightNodeSetEvents;

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
                    ParseOutcome::Parsed(ColorlightNodeSetEvents::Color(value))
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
                    ParseOutcome::Parsed(ColorlightNodeSetEvents::ColorTemperature(value))
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

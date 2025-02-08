use homie5::{
    device_description::{
        ColorFormat, HomieDeviceDescription, HomieNodeDescription, HomiePropertyFormat,
        IntegerRange, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
    Homie5DeviceProtocol, Homie5Message, HomieColorValue, HomieID, HomieValue, NodeRef,
    PropertyRef,
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_TYPE_COLORLIGHT;

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
    pub fn match_parse(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> Option<ColorlightNodeSetEvents> {
        if property.match_with_node(&self.node, &self.color_prop_id) {
            desc.with_property(property, |prop_desc| {
                if let Ok(HomieValue::Color(value)) = HomieValue::parse(set_value, prop_desc) {
                    Some(ColorlightNodeSetEvents::Color(value))
                } else {
                    None
                }
            })?
        } else if property.match_with_node(&self.node, &self.color_temp_prop_id) {
            desc.with_property(property, |prop_desc| {
                if let Ok(HomieValue::Integer(value)) = HomieValue::parse(set_value, prop_desc) {
                    Some(ColorlightNodeSetEvents::ColorTemperature(value))
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
    ) -> Option<ColorlightNodeSetEvents> {
        match event {
            Homie5Message::PropertySet {
                property,
                set_value,
            } => self.match_parse(property, desc, set_value),
            _ => None,
        }
    }
}

use homie5::{
    Homie5DeviceProtocol, Homie5Message, HomieID, HomieValue, NodeRef, PropertyRef,
    device_description::{
        BooleanFormat, FloatRange, HomieDeviceDescription, HomieNodeDescription,
        HomiePropertyFormat, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_CAP_CAMERA, SetCommandParser};

pub const CAMERA_NODE_DEFAULT_ID: HomieID = HomieID::new_const("camera");
pub const CAMERA_NODE_DEFAULT_NAME: &str = "Camera";
pub const CAMERA_NODE_STREAM_URL_PROP_ID: HomieID = HomieID::new_const("stream-url");
pub const CAMERA_NODE_SNAPSHOT_URL_PROP_ID: HomieID = HomieID::new_const("snapshot-url");
pub const CAMERA_NODE_RECORDING_PROP_ID: HomieID = HomieID::new_const("recording");
pub const CAMERA_NODE_MOTION_DETECTED_PROP_ID: HomieID = HomieID::new_const("motion-detected");
pub const CAMERA_NODE_OBJECT_DETECTED_PROP_ID: HomieID = HomieID::new_const("object-detected");
pub const CAMERA_NODE_PAN_PROP_ID: HomieID = HomieID::new_const("pan");
pub const CAMERA_NODE_TILT_PROP_ID: HomieID = HomieID::new_const("tilt");
pub const CAMERA_NODE_ZOOM_PROP_ID: HomieID = HomieID::new_const("zoom");

// ── Node (state) ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct CameraNode {
    pub publisher: CameraNodePublisher,
    pub stream_url: String,
    pub snapshot_url: Option<String>,
    pub recording: Option<bool>,
    pub motion_detected: Option<bool>,
    pub object_detected: Option<String>,
    pub pan: Option<f64>,
    pub tilt: Option<f64>,
    pub zoom: Option<f64>,
}

// ── Set events ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum CameraNodeSetEvents {
    Recording(bool),
    Pan(f64),
    Tilt(f64),
    Zoom(f64),
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraNodeConfig {
    pub snapshot: bool,
    pub recording: bool,
    pub motion_detected: bool,
    pub object_detected: bool,
    pub pan: bool,
    pub tilt: bool,
    pub zoom: bool,
    pub zoom_max: f64,
}

impl Default for CameraNodeConfig {
    fn default() -> Self {
        Self {
            snapshot: true,
            recording: false,
            motion_detected: false,
            object_detected: false,
            pan: false,
            tilt: false,
            zoom: false,
            zoom_max: 10.0,
        }
    }
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct CameraNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl CameraNodeBuilder {
    pub fn new(config: &CameraNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(CAMERA_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_CAMERA);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &CameraNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            CAMERA_NODE_STREAM_URL_PROP_ID,
            PropertyDescriptionBuilder::new(homie5::HomieDataType::String)
                .name("Stream URL")
                .settable(false)
                .retained(true)
                .build(),
        )
        .add_property_cond(CAMERA_NODE_SNAPSHOT_URL_PROP_ID, config.snapshot, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::String)
                .name("Snapshot URL")
                .settable(false)
                .retained(true)
                .build()
        })
        .add_property_cond(CAMERA_NODE_RECORDING_PROP_ID, config.recording, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                .name("Recording")
                .format(HomiePropertyFormat::Boolean(BooleanFormat {
                    false_val: "stopped".to_owned(),
                    true_val: "recording".to_owned(),
                }))
                .settable(true)
                .retained(true)
                .build()
        })
        .add_property_cond(
            CAMERA_NODE_MOTION_DETECTED_PROP_ID,
            config.motion_detected,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                    .name("Motion detected")
                    .format(HomiePropertyFormat::Boolean(BooleanFormat {
                        false_val: "no motion".to_owned(),
                        true_val: "motion detected".to_owned(),
                    }))
                    .settable(false)
                    .retained(true)
                    .build()
            },
        )
        .add_property_cond(
            CAMERA_NODE_OBJECT_DETECTED_PROP_ID,
            config.object_detected,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::String)
                    .name("Object detected")
                    .settable(false)
                    .retained(true)
                    .build()
            },
        )
        .add_property_cond(CAMERA_NODE_PAN_PROP_ID, config.pan, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                .name("Pan angle")
                .unit("°")
                .format(HomiePropertyFormat::FloatRange(FloatRange {
                    min: Some(-180.0),
                    max: Some(180.0),
                    step: None,
                }))
                .settable(true)
                .retained(true)
                .build()
        })
        .add_property_cond(CAMERA_NODE_TILT_PROP_ID, config.tilt, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                .name("Tilt angle")
                .unit("°")
                .format(HomiePropertyFormat::FloatRange(FloatRange {
                    min: Some(-90.0),
                    max: Some(90.0),
                    step: None,
                }))
                .settable(true)
                .retained(true)
                .build()
        })
        .add_property_cond(CAMERA_NODE_ZOOM_PROP_ID, config.zoom, || {
            PropertyDescriptionBuilder::new(homie5::HomieDataType::Float)
                .name("Zoom level")
                .format(HomiePropertyFormat::FloatRange(FloatRange {
                    min: Some(1.0),
                    max: Some(config.zoom_max),
                    step: None,
                }))
                .settable(true)
                .retained(true)
                .build()
        })
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
    ) -> (HomieNodeDescription, CameraNodePublisher) {
        (
            self.node_builder.build(),
            CameraNodePublisher::new(
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
pub struct CameraNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    stream_url_prop: HomieID,
    snapshot_url_prop: HomieID,
    recording_prop: HomieID,
    motion_detected_prop: HomieID,
    object_detected_prop: HomieID,
    pan_prop: HomieID,
    tilt_prop: HomieID,
    zoom_prop: HomieID,
}

impl CameraNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            stream_url_prop: CAMERA_NODE_STREAM_URL_PROP_ID,
            snapshot_url_prop: CAMERA_NODE_SNAPSHOT_URL_PROP_ID,
            recording_prop: CAMERA_NODE_RECORDING_PROP_ID,
            motion_detected_prop: CAMERA_NODE_MOTION_DETECTED_PROP_ID,
            object_detected_prop: CAMERA_NODE_OBJECT_DETECTED_PROP_ID,
            pan_prop: CAMERA_NODE_PAN_PROP_ID,
            tilt_prop: CAMERA_NODE_TILT_PROP_ID,
            zoom_prop: CAMERA_NODE_ZOOM_PROP_ID,
        }
    }

    pub fn stream_url(&self, value: &str) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.stream_url_prop, value, true)
    }

    pub fn snapshot_url(&self, value: &str) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.snapshot_url_prop, value, true)
    }

    pub fn recording(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.recording_prop,
            value.to_string(),
            true,
        )
    }

    pub fn motion_detected(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.motion_detected_prop,
            value.to_string(),
            true,
        )
    }

    pub fn object_detected(&self, value: &str) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.object_detected_prop, value, true)
    }

    pub fn pan(&self, value: f64) -> homie5::client::Publish {
        self.client
            .publish_value(self.node.node_id(), &self.pan_prop, value.to_string(), true)
    }

    pub fn tilt(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.tilt_prop,
            value.to_string(),
            true,
        )
    }

    pub fn zoom(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.zoom_prop,
            value.to_string(),
            true,
        )
    }
}

impl SetCommandParser for CameraNodePublisher {
    type Event = CameraNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &self.recording_prop) {
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
                    ParseOutcome::Parsed(CameraNodeSetEvents::Recording(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &self.pan_prop) {
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
                Ok(HomieValue::Float(value)) => {
                    ParseOutcome::Parsed(CameraNodeSetEvents::Pan(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &self.tilt_prop) {
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
                Ok(HomieValue::Float(value)) => {
                    ParseOutcome::Parsed(CameraNodeSetEvents::Tilt(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property.prop_id().to_string(),
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &self.zoom_prop) {
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
                Ok(HomieValue::Float(value)) => {
                    ParseOutcome::Parsed(CameraNodeSetEvents::Zoom(value))
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
                self.stream_url_prop.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

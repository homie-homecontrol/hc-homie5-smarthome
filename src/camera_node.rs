use homie5::{
    Homie5DeviceProtocol, Homie5Message, HomieID, HomieValue, NodeRef, PropertyRef,
    device_description::{
        FloatRange, HomieDeviceDescription, HomieNodeDescription, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, PropSpec, SMARTHOME_CAP_CAMERA, SetCommandParser,
};

pub const CAMERA_NODE_DEFAULT_ID: HomieID = HomieID::new_const("camera");
pub const CAMERA_NODE_DEFAULT_NAME: &str = "Camera";

pub const CAMERA_NODE_STREAM_URL_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("stream-url"),
    retained: true,
    settable: false,
};
pub const CAMERA_NODE_SNAPSHOT_URL_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("snapshot-url"),
    retained: true,
    settable: false,
};
pub const CAMERA_NODE_RECORDING_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("recording"),
    retained: true,
    settable: true,
};
pub const CAMERA_NODE_MOTION_DETECTED_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("motion-detected"),
    retained: true,
    settable: false,
};
pub const CAMERA_NODE_OBJECT_DETECTED_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("object-detected"),
    retained: true,
    settable: false,
};
pub const CAMERA_NODE_PAN_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("pan"),
    retained: true,
    settable: true,
};
pub const CAMERA_NODE_TILT_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("tilt"),
    retained: true,
    settable: true,
};
pub const CAMERA_NODE_ZOOM_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("zoom"),
    retained: true,
    settable: true,
};

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
            CAMERA_NODE_STREAM_URL_PROP.id,
            PropertyDescriptionBuilder::string()
                .name("Stream URL")
                .settable(CAMERA_NODE_STREAM_URL_PROP.settable)
                .retained(CAMERA_NODE_STREAM_URL_PROP.retained)
                .build(),
        )
        .add_property_cond(CAMERA_NODE_SNAPSHOT_URL_PROP.id, config.snapshot, || {
            PropertyDescriptionBuilder::string()
                .name("Snapshot URL")
                .settable(CAMERA_NODE_SNAPSHOT_URL_PROP.settable)
                .retained(CAMERA_NODE_SNAPSHOT_URL_PROP.retained)
                .build()
        })
        .add_property_cond(CAMERA_NODE_RECORDING_PROP.id, config.recording, || {
            PropertyDescriptionBuilder::boolean()
                .name("Recording")
                .boolean_labels("stopped", "recording")
                .settable(CAMERA_NODE_RECORDING_PROP.settable)
                .retained(CAMERA_NODE_RECORDING_PROP.retained)
                .build()
        })
        .add_property_cond(
            CAMERA_NODE_MOTION_DETECTED_PROP.id,
            config.motion_detected,
            || {
                PropertyDescriptionBuilder::boolean()
                    .name("Motion detected")
                    .boolean_labels("no motion", "motion detected")
                    .settable(CAMERA_NODE_MOTION_DETECTED_PROP.settable)
                    .retained(CAMERA_NODE_MOTION_DETECTED_PROP.retained)
                    .build()
            },
        )
        .add_property_cond(
            CAMERA_NODE_OBJECT_DETECTED_PROP.id,
            config.object_detected,
            || {
                PropertyDescriptionBuilder::string()
                    .name("Object detected")
                    .settable(CAMERA_NODE_OBJECT_DETECTED_PROP.settable)
                    .retained(CAMERA_NODE_OBJECT_DETECTED_PROP.retained)
                    .build()
            },
        )
        .add_property_cond(CAMERA_NODE_PAN_PROP.id, config.pan, || {
            PropertyDescriptionBuilder::float()
                .name("Pan angle")
                .unit("°")
                .float_range(FloatRange {
                    min: Some(-180.0),
                    max: Some(180.0),
                    step: None,
                })
                .settable(CAMERA_NODE_PAN_PROP.settable)
                .retained(CAMERA_NODE_PAN_PROP.retained)
                .build()
        })
        .add_property_cond(CAMERA_NODE_TILT_PROP.id, config.tilt, || {
            PropertyDescriptionBuilder::float()
                .name("Tilt angle")
                .unit("°")
                .float_range(FloatRange {
                    min: Some(-90.0),
                    max: Some(90.0),
                    step: None,
                })
                .settable(CAMERA_NODE_TILT_PROP.settable)
                .retained(CAMERA_NODE_TILT_PROP.retained)
                .build()
        })
        .add_property_cond(CAMERA_NODE_ZOOM_PROP.id, config.zoom, || {
            PropertyDescriptionBuilder::float()
                .name("Zoom level")
                .float_range(FloatRange {
                    min: Some(1.0),
                    max: Some(config.zoom_max),
                    step: None,
                })
                .settable(CAMERA_NODE_ZOOM_PROP.settable)
                .retained(CAMERA_NODE_ZOOM_PROP.retained)
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
}

impl CameraNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn stream_url(&self, value: &str) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &CAMERA_NODE_STREAM_URL_PROP.id,
            value,
            CAMERA_NODE_STREAM_URL_PROP.retained,
        )
    }

    pub fn snapshot_url(&self, value: &str) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &CAMERA_NODE_SNAPSHOT_URL_PROP.id,
            value,
            CAMERA_NODE_SNAPSHOT_URL_PROP.retained,
        )
    }

    pub fn recording(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &CAMERA_NODE_RECORDING_PROP.id,
            value.to_string(),
            CAMERA_NODE_RECORDING_PROP.retained,
        )
    }

    pub fn motion_detected(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &CAMERA_NODE_MOTION_DETECTED_PROP.id,
            value.to_string(),
            CAMERA_NODE_MOTION_DETECTED_PROP.retained,
        )
    }

    pub fn object_detected(&self, value: &str) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &CAMERA_NODE_OBJECT_DETECTED_PROP.id,
            value,
            CAMERA_NODE_OBJECT_DETECTED_PROP.retained,
        )
    }

    pub fn pan(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &CAMERA_NODE_PAN_PROP.id,
            value.to_string(),
            CAMERA_NODE_PAN_PROP.retained,
        )
    }

    pub fn tilt(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &CAMERA_NODE_TILT_PROP.id,
            value.to_string(),
            CAMERA_NODE_TILT_PROP.retained,
        )
    }

    pub fn zoom(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &CAMERA_NODE_ZOOM_PROP.id,
            value.to_string(),
            CAMERA_NODE_ZOOM_PROP.retained,
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

        if property.match_with_node(&self.node, &CAMERA_NODE_RECORDING_PROP.id) {
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
        } else if property.match_with_node(&self.node, &CAMERA_NODE_PAN_PROP.id) {
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
        } else if property.match_with_node(&self.node, &CAMERA_NODE_TILT_PROP.id) {
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
        } else if property.match_with_node(&self.node, &CAMERA_NODE_ZOOM_PROP.id) {
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
                CAMERA_NODE_STREAM_URL_PROP.id.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

use std::{fmt, str::FromStr};

use homie5::{
    Homie5DeviceProtocol, Homie5ProtocolError, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{PropSpec, SMARTHOME_CAP_GESTURE};

pub const GESTURE_NODE_DEFAULT_ID: HomieID = HomieID::new_const("gesture");
pub const GESTURE_NODE_DEFAULT_NAME: &str = "Gesture";

pub const GESTURE_NODE_GESTURE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("gesture"),
    retained: false,
    settable: false,
};

// ── Gesture vocabulary ──────────────────────────────────────────────────────

/// Named gestures: moving the device itself, touch-surface gestures and mid-air
/// gestures. The list is closed; new gestures are added by a spec change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gesture {
    Shake,
    Throw,
    FreeFall,
    Slide,
    Tap,
    Flip90,
    Flip180,
    SwipeUp,
    SwipeDown,
    SwipeLeft,
    SwipeRight,
    WaveLeft,
    WaveRight,
}

impl Gesture {
    pub const ALL: [Gesture; 13] = [
        Gesture::Shake,
        Gesture::Throw,
        Gesture::FreeFall,
        Gesture::Slide,
        Gesture::Tap,
        Gesture::Flip90,
        Gesture::Flip180,
        Gesture::SwipeUp,
        Gesture::SwipeDown,
        Gesture::SwipeLeft,
        Gesture::SwipeRight,
        Gesture::WaveLeft,
        Gesture::WaveRight,
    ];

    pub const fn as_str(&self) -> &'static str {
        match self {
            Gesture::Shake => "shake",
            Gesture::Throw => "throw",
            Gesture::FreeFall => "free-fall",
            Gesture::Slide => "slide",
            Gesture::Tap => "tap",
            Gesture::Flip90 => "flip-90",
            Gesture::Flip180 => "flip-180",
            Gesture::SwipeUp => "swipe-up",
            Gesture::SwipeDown => "swipe-down",
            Gesture::SwipeLeft => "swipe-left",
            Gesture::SwipeRight => "swipe-right",
            Gesture::WaveLeft => "wave-left",
            Gesture::WaveRight => "wave-right",
        }
    }
}

impl fmt::Display for Gesture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Gesture {
    type Err = Homie5ProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|gesture| gesture.as_str() == s)
            .ok_or(Homie5ProtocolError::InvalidPayload)
    }
}

impl Serialize for Gesture {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Gesture {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        value
            .parse()
            .map_err(|_| serde::de::Error::custom(format!("invalid gesture: {value}")))
    }
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GestureNodeConfig {
    /// The gestures this device can produce; becomes the `gesture` enum format.
    pub gestures: Vec<Gesture>,
}

impl Default for GestureNodeConfig {
    fn default() -> Self {
        Self {
            gestures: Gesture::ALL.to_vec(),
        }
    }
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct GestureNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl GestureNodeBuilder {
    pub fn new(config: &GestureNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(GESTURE_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_GESTURE);

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &GestureNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            GESTURE_NODE_GESTURE_PROP.id,
            PropertyDescriptionBuilder::enumeration(config.gestures.iter().map(|g| g.as_str()))
                .unwrap()
                .name("Gesture event")
                .settable(GESTURE_NODE_GESTURE_PROP.settable)
                .retained(GESTURE_NODE_GESTURE_PROP.retained)
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
    ) -> (HomieNodeDescription, GestureNodePublisher) {
        (
            self.node_builder.build(),
            GestureNodePublisher::new(
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

impl Default for GestureNodeBuilder {
    fn default() -> Self {
        Self::new(&GestureNodeConfig::default())
    }
}

// ── Publisher ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct GestureNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
}

impl GestureNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn gesture(&self, gesture: Gesture) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &GESTURE_NODE_GESTURE_PROP.id,
            gesture.as_str(),
            GESTURE_NODE_GESTURE_PROP.retained,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use homie5::{HomieDomain, client::QoS, device_description::HomiePropertyFormat};

    #[test]
    fn every_gesture_round_trips_through_its_wire_string() {
        for gesture in Gesture::ALL {
            assert_eq!(gesture.as_str().parse::<Gesture>().unwrap(), gesture);
            let json = serde_json::to_string(&gesture).unwrap();
            assert_eq!(serde_json::from_str::<Gesture>(&json).unwrap(), gesture);
        }
        assert!("rotate-cw".parse::<Gesture>().is_err());
    }

    #[test]
    fn description_lists_the_configured_gestures() {
        let config = GestureNodeConfig {
            gestures: vec![Gesture::Shake, Gesture::Flip90],
        };
        let node = GestureNodeBuilder::new(&config).build();
        let gesture = node.properties.get(&GESTURE_NODE_GESTURE_PROP.id).unwrap();

        assert_eq!(node.r#type.as_deref(), Some(SMARTHOME_CAP_GESTURE));
        assert!(!gesture.retained);
        assert!(!gesture.settable);
        assert_eq!(
            gesture.format,
            HomiePropertyFormat::Enum(vec!["shake".to_string(), "flip-90".to_string()])
        );
    }

    #[test]
    fn default_config_declares_every_gesture() {
        assert_eq!(GestureNodeConfig::default().gestures, Gesture::ALL.to_vec());
    }

    #[test]
    fn gesture_is_published_non_retained() {
        let (client, _) =
            Homie5DeviceProtocol::new(HomieID::new_const("dev"), HomieDomain::Default);
        let (_, publisher) =
            GestureNodeBuilder::default().build_with_publisher(GESTURE_NODE_DEFAULT_ID, &client);

        let publish = publisher.gesture(Gesture::FreeFall);
        assert!(publish.topic.ends_with("/gesture/gesture"));
        assert_eq!(publish.payload, b"free-fall");
        assert!(!publish.retain);
        assert_eq!(publish.qos, QoS::AtMostOnce);
    }
}

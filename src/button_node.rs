use std::{fmt::Display, str::FromStr};

use homie5::{
    Homie5DeviceProtocol, Homie5ProtocolError, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{PropSpec, SMARTHOME_CAP_BUTTON};

pub const BUTTON_NODE_DEFAULT_ID: HomieID = HomieID::new_const("button");
pub const BUTTON_NODE_DEFAULT_NAME: &str = "Pushbutton";

pub const BUTTON_NODE_ACTION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("action"),
    retained: false,
    settable: false,
};

/// Button interaction events, following the press sequence of the Matter Switch
/// cluster and the Home Assistant standard button event types.
///
/// A device that reports only one event per click publishes `PressEnd` and never
/// invents a `PressStart`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ButtonNodeActions {
    /// The button was pressed down.
    PressStart,
    /// The button was released after a short press ("the click").
    PressEnd,
    /// The button was held past the long-press threshold.
    LongPressStart,
    /// The button is still held; repeated at a device-defined interval.
    LongPressRepeat,
    /// The button was released after a long press.
    LongPressEnd,
    /// A sequence of two presses completed.
    MultiPress2,
    /// A sequence of three presses completed.
    MultiPress3,
    /// A sequence of four presses completed.
    MultiPress4,
    /// A sequence of five presses completed.
    MultiPress5,
    /// A sequence of six presses completed.
    MultiPress6,
}

impl ButtonNodeActions {
    pub const ALL: [ButtonNodeActions; 10] = [
        ButtonNodeActions::PressStart,
        ButtonNodeActions::PressEnd,
        ButtonNodeActions::LongPressStart,
        ButtonNodeActions::LongPressRepeat,
        ButtonNodeActions::LongPressEnd,
        ButtonNodeActions::MultiPress2,
        ButtonNodeActions::MultiPress3,
        ButtonNodeActions::MultiPress4,
        ButtonNodeActions::MultiPress5,
        ButtonNodeActions::MultiPress6,
    ];

    pub const fn as_str(&self) -> &'static str {
        match self {
            ButtonNodeActions::PressStart => "press-start",
            ButtonNodeActions::PressEnd => "press-end",
            ButtonNodeActions::LongPressStart => "long-press-start",
            ButtonNodeActions::LongPressRepeat => "long-press-repeat",
            ButtonNodeActions::LongPressEnd => "long-press-end",
            ButtonNodeActions::MultiPress2 => "multi-press-2",
            ButtonNodeActions::MultiPress3 => "multi-press-3",
            ButtonNodeActions::MultiPress4 => "multi-press-4",
            ButtonNodeActions::MultiPress5 => "multi-press-5",
            ButtonNodeActions::MultiPress6 => "multi-press-6",
        }
    }

    /// The action for a completed sequence of `count` presses, if the vocabulary
    /// covers that count (2 to 6).
    pub const fn multi_press(count: u64) -> Option<Self> {
        match count {
            2 => Some(ButtonNodeActions::MultiPress2),
            3 => Some(ButtonNodeActions::MultiPress3),
            4 => Some(ButtonNodeActions::MultiPress4),
            5 => Some(ButtonNodeActions::MultiPress5),
            6 => Some(ButtonNodeActions::MultiPress6),
            _ => None,
        }
    }

    pub fn all_variants() -> &'static [Self] {
        &Self::ALL
    }

    pub fn to_string_vec() -> Vec<String> {
        Self::all_variants().iter().map(|v| v.to_string()).collect()
    }
}

impl FromStr for ButtonNodeActions {
    type Err = Homie5ProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|action| action.as_str() == s)
            .ok_or(Homie5ProtocolError::InvalidPayload)
    }
}

impl Display for ButtonNodeActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&ButtonNodeActions> for &'static str {
    fn from(action: &ButtonNodeActions) -> Self {
        action.as_str()
    }
}

impl Serialize for ButtonNodeActions {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ButtonNodeActions {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        value
            .parse()
            .map_err(|_| serde::de::Error::custom(format!("invalid button action: {value}")))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ButtonNodeConfig {
    /// The actions this button can produce; becomes the `action` enum format.
    pub actions: Vec<ButtonNodeActions>,
}

impl Default for ButtonNodeConfig {
    fn default() -> Self {
        Self {
            actions: vec![ButtonNodeActions::PressEnd],
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
        .r#type(SMARTHOME_CAP_BUTTON);

        Self { node_builder: db }
    }

    fn build_node(db: NodeDescriptionBuilder, config: &ButtonNodeConfig) -> NodeDescriptionBuilder {
        db.add_property(
            BUTTON_NODE_ACTION_PROP.id,
            PropertyDescriptionBuilder::enumeration(config.actions.iter().map(|a| a.as_str()))
                .unwrap()
                .name("Button action event")
                .settable(BUTTON_NODE_ACTION_PROP.settable)
                .retained(BUTTON_NODE_ACTION_PROP.retained)
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
}

impl ButtonNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self { node, client }
    }

    pub fn action(&self, kind: &ButtonNodeActions) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &BUTTON_NODE_ACTION_PROP.id,
            kind.as_str(),
            BUTTON_NODE_ACTION_PROP.retained,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use homie5::{HomieDomain, client::QoS, device_description::HomiePropertyFormat};

    #[test]
    fn every_action_round_trips_through_its_wire_string() {
        for action in ButtonNodeActions::ALL {
            assert_eq!(
                action.as_str().parse::<ButtonNodeActions>().unwrap(),
                action
            );
            let json = serde_json::to_string(&action).unwrap();
            assert_eq!(json, format!("\"{}\"", action.as_str()));
            assert_eq!(
                serde_json::from_str::<ButtonNodeActions>(&json).unwrap(),
                action
            );
        }
    }

    #[test]
    fn previous_vocabulary_is_rejected() {
        for old in [
            "press",
            "long-press",
            "double-press",
            "release",
            "long-release",
            "continuous",
        ] {
            assert!(
                old.parse::<ButtonNodeActions>().is_err(),
                "{old} must not parse"
            );
        }
    }

    #[test]
    fn multi_press_covers_two_to_six_presses() {
        assert_eq!(ButtonNodeActions::multi_press(1), None);
        assert_eq!(
            ButtonNodeActions::multi_press(2),
            Some(ButtonNodeActions::MultiPress2)
        );
        assert_eq!(
            ButtonNodeActions::multi_press(6),
            Some(ButtonNodeActions::MultiPress6)
        );
        assert_eq!(ButtonNodeActions::multi_press(7), None);
    }

    #[test]
    fn description_lists_the_configured_actions() {
        let config = ButtonNodeConfig {
            actions: vec![
                ButtonNodeActions::PressEnd,
                ButtonNodeActions::LongPressStart,
                ButtonNodeActions::MultiPress2,
            ],
        };
        let node = ButtonNodeBuilder::new(&config).build();
        let action = node.properties.get(&BUTTON_NODE_ACTION_PROP.id).unwrap();

        assert_eq!(node.r#type.as_deref(), Some(SMARTHOME_CAP_BUTTON));
        assert!(!action.retained);
        assert!(!action.settable);
        assert_eq!(
            action.format,
            HomiePropertyFormat::Enum(vec![
                "press-end".to_string(),
                "long-press-start".to_string(),
                "multi-press-2".to_string(),
            ])
        );
    }

    #[test]
    fn config_deserializes_new_action_names_and_defaults_to_press_end() {
        let config: ButtonNodeConfig =
            serde_json::from_str(r#"{"actions":["press-start","long-press-repeat"]}"#).unwrap();
        assert_eq!(
            config.actions,
            vec![
                ButtonNodeActions::PressStart,
                ButtonNodeActions::LongPressRepeat
            ]
        );
        assert_eq!(
            ButtonNodeConfig::default().actions,
            vec![ButtonNodeActions::PressEnd]
        );
        assert!(serde_json::from_str::<ButtonNodeConfig>(r#"{"actions":["press"]}"#).is_err());
    }

    #[test]
    fn action_is_published_non_retained() {
        let (client, _) =
            Homie5DeviceProtocol::new(HomieID::new_const("dev"), HomieDomain::Default);
        let (_, publisher) =
            ButtonNodeBuilder::default().build_with_publisher(BUTTON_NODE_DEFAULT_ID, &client);

        let publish = publisher.action(&ButtonNodeActions::LongPressRepeat);
        assert!(publish.topic.ends_with("/button/action"));
        assert_eq!(publish.payload, b"long-press-repeat");
        assert!(!publish.retain);
        assert_eq!(publish.qos, QoS::AtMostOnce);
    }
}

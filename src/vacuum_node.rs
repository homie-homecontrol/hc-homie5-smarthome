use std::{collections::BTreeMap, fmt, str::FromStr};

use homie5::{
    HOMIE_UNIT_PERCENT, Homie5DeviceProtocol, Homie5Message, Homie5ProtocolError, HomieID,
    HomieValue, NodeRef, PropertyRef,
    device_description::{
        HomieDeviceDescription, HomieNodeDescription, IntegerRange, NodeDescriptionBuilder,
        PropertyDescriptionBuilder, PropertyDescriptionValidationError,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    ParseError, ParseErrorKind, ParseOutcome, PropSpec, SMARTHOME_CAP_VACUUM, SetCommandParser,
};

pub const VACUUM_NODE_DEFAULT_ID: HomieID = HomieID::new_const("vacuum");
pub const VACUUM_NODE_DEFAULT_NAME: &str = "Vacuum";

pub const VACUUM_NODE_STATE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("state"),
    retained: true,
    settable: false,
};
pub const VACUUM_NODE_ACTION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("action"),
    retained: false,
    settable: true,
};
pub const VACUUM_NODE_SUCTION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("suction"),
    retained: true,
    settable: true,
};
/// Current error text; empty while the vacuum reports no error.
pub const VACUUM_NODE_ERROR_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("error"),
    retained: true,
    settable: false,
};
pub const VACUUM_NODE_MAIN_BRUSH_LIFE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("main-brush-life"),
    retained: true,
    settable: false,
};
pub const VACUUM_NODE_SIDE_BRUSH_LIFE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("side-brush-life"),
    retained: true,
    settable: false,
};
pub const VACUUM_NODE_FILTER_LIFE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("filter-life"),
    retained: true,
    settable: false,
};
pub const VACUUM_NODE_SENSOR_LIFE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("sensor-life"),
    retained: true,
    settable: false,
};
pub const VACUUM_NODE_MOP_LIFE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("mop-life"),
    retained: true,
    settable: false,
};
/// Resets the usage counter of one consumable after it was replaced or cleaned.
pub const VACUUM_NODE_RESET_CONSUMABLE_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("reset-consumable"),
    retained: false,
    settable: true,
};

// ── Enums ───────────────────────────────────────────────────────────────────

/// Coarse, vendor-neutral runtime state of a vacuum robot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VacuumState {
    Idle,
    Cleaning,
    Paused,
    Returning,
    Docked,
    Error,
}

impl VacuumState {
    pub const ALL: [VacuumState; 6] = [
        VacuumState::Idle,
        VacuumState::Cleaning,
        VacuumState::Paused,
        VacuumState::Returning,
        VacuumState::Docked,
        VacuumState::Error,
    ];

    pub const fn as_str(&self) -> &'static str {
        match self {
            VacuumState::Idle => "idle",
            VacuumState::Cleaning => "cleaning",
            VacuumState::Paused => "paused",
            VacuumState::Returning => "returning",
            VacuumState::Docked => "docked",
            VacuumState::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VacuumAction {
    /// Start cleaning, or resume a paused run.
    Start,
    Pause,
    Stop,
    /// Return to the dock.
    Dock,
    /// Clean the area around the current position.
    Spot,
    /// Make the robot announce its position.
    Locate,
}

impl VacuumAction {
    pub const ALL: [VacuumAction; 6] = [
        VacuumAction::Start,
        VacuumAction::Pause,
        VacuumAction::Stop,
        VacuumAction::Dock,
        VacuumAction::Spot,
        VacuumAction::Locate,
    ];

    pub const fn as_str(&self) -> &'static str {
        match self {
            VacuumAction::Start => "start",
            VacuumAction::Pause => "pause",
            VacuumAction::Stop => "stop",
            VacuumAction::Dock => "dock",
            VacuumAction::Spot => "spot",
            VacuumAction::Locate => "locate",
        }
    }
}

/// A consumable whose remaining life the vacuum reports. Each one configured
/// gets a `<id>-life` property and a `reset-consumable` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VacuumConsumable {
    MainBrush,
    SideBrush,
    Filter,
    Sensor,
    Mop,
}

impl VacuumConsumable {
    pub const ALL: [VacuumConsumable; 5] = [
        VacuumConsumable::MainBrush,
        VacuumConsumable::SideBrush,
        VacuumConsumable::Filter,
        VacuumConsumable::Sensor,
        VacuumConsumable::Mop,
    ];

    pub const fn as_str(&self) -> &'static str {
        match self {
            VacuumConsumable::MainBrush => "main-brush",
            VacuumConsumable::SideBrush => "side-brush",
            VacuumConsumable::Filter => "filter",
            VacuumConsumable::Sensor => "sensor",
            VacuumConsumable::Mop => "mop",
        }
    }

    /// The `<id>-life` property of this consumable.
    pub const fn life_prop(&self) -> PropSpec {
        match self {
            VacuumConsumable::MainBrush => VACUUM_NODE_MAIN_BRUSH_LIFE_PROP,
            VacuumConsumable::SideBrush => VACUUM_NODE_SIDE_BRUSH_LIFE_PROP,
            VacuumConsumable::Filter => VACUUM_NODE_FILTER_LIFE_PROP,
            VacuumConsumable::Sensor => VACUUM_NODE_SENSOR_LIFE_PROP,
            VacuumConsumable::Mop => VACUUM_NODE_MOP_LIFE_PROP,
        }
    }

    const fn life_name(&self) -> &'static str {
        match self {
            VacuumConsumable::MainBrush => "Main brush life",
            VacuumConsumable::SideBrush => "Side brush life",
            VacuumConsumable::Filter => "Filter life",
            VacuumConsumable::Sensor => "Sensor life",
            VacuumConsumable::Mop => "Mop life",
        }
    }
}

/// Implements `FromStr`, `Display` and canonical-string serde for a vacuum enum.
macro_rules! vacuum_wire_enum {
    ($ty:ident, $what:literal) => {
        impl FromStr for $ty {
            type Err = Homie5ProtocolError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::ALL
                    .into_iter()
                    .find(|v| v.as_str() == s)
                    .ok_or(Homie5ProtocolError::InvalidPayload)
            }
        }

        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl Serialize for $ty {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = String::deserialize(deserializer)?;
                value.parse().map_err(|_| {
                    serde::de::Error::custom(format!(concat!("invalid ", $what, ": {}"), value))
                })
            }
        }
    };
}

vacuum_wire_enum!(VacuumState, "vacuum state");
vacuum_wire_enum!(VacuumAction, "vacuum action");
vacuum_wire_enum!(VacuumConsumable, "vacuum consumable");

// ── Node (state) ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct VacuumNode {
    pub publisher: VacuumNodePublisher,
    pub state: VacuumState,
    pub suction: Option<String>,
    pub error: Option<String>,
    /// Remaining life in percent per configured consumable.
    pub consumable_life: BTreeMap<VacuumConsumable, i64>,
}

// ── Set events ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VacuumNodeSetEvents {
    Action(VacuumAction),
    Suction(String),
    ResetConsumable(VacuumConsumable),
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VacuumNodeConfig {
    /// The actions the vacuum supports; becomes the `action` enum format.
    pub actions: Vec<VacuumAction>,
    /// Suction presets in device order; `suction` is omitted when empty.
    pub suction: Vec<String>,
    /// Whether the node has the `error` property.
    pub error: bool,
    /// Consumables with a `<id>-life` property; `reset-consumable` is omitted when empty.
    pub consumables: Vec<VacuumConsumable>,
}

impl Default for VacuumNodeConfig {
    fn default() -> Self {
        Self {
            actions: VacuumAction::ALL.to_vec(),
            suction: Vec::new(),
            error: false,
            consumables: Vec::new(),
        }
    }
}

impl VacuumNodeConfig {
    pub fn validate(&self) -> Result<(), VacuumConfigError> {
        PropertyDescriptionBuilder::enumeration(self.actions.iter().map(VacuumAction::as_str))
            .map_err(VacuumConfigError::InvalidActions)?;
        if !self.suction.is_empty() {
            PropertyDescriptionBuilder::enumeration(self.suction.iter().map(String::as_str))
                .map_err(VacuumConfigError::InvalidSuction)?;
        }
        if !self.consumables.is_empty() {
            PropertyDescriptionBuilder::enumeration(
                self.consumables.iter().map(VacuumConsumable::as_str),
            )
            .map_err(VacuumConfigError::InvalidConsumables)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VacuumConfigError {
    /// `actions` is empty or lists an action twice.
    InvalidActions(PropertyDescriptionValidationError),
    /// `suction` contains an empty or duplicate preset name.
    InvalidSuction(PropertyDescriptionValidationError),
    /// `consumables` lists a consumable twice.
    InvalidConsumables(PropertyDescriptionValidationError),
}

impl fmt::Display for VacuumConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VacuumConfigError::InvalidActions(err) => write!(f, "invalid vacuum actions: {err}"),
            VacuumConfigError::InvalidSuction(err) => {
                write!(f, "invalid vacuum suction presets: {err}")
            }
            VacuumConfigError::InvalidConsumables(err) => {
                write!(f, "invalid vacuum consumables: {err}")
            }
        }
    }
}

impl std::error::Error for VacuumConfigError {}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct VacuumNodeBuilder {
    config: VacuumNodeConfig,
    node_builder: NodeDescriptionBuilder,
}

impl VacuumNodeBuilder {
    /// Rejects a config whose actions, suction presets or consumables are not a
    /// valid Homie enum (no actions, or empty or duplicate values).
    pub fn new(config: &VacuumNodeConfig) -> Result<Self, VacuumConfigError> {
        config.validate()?;
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(VACUUM_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_VACUUM);

        Ok(Self {
            config: config.clone(),
            node_builder: db,
        })
    }

    fn build_node(db: NodeDescriptionBuilder, config: &VacuumNodeConfig) -> NodeDescriptionBuilder {
        let mut db = db
            .add_property(
                VACUUM_NODE_STATE_PROP.id,
                PropertyDescriptionBuilder::enumeration(
                    VacuumState::ALL.iter().map(|s| s.as_str()),
                )
                .unwrap()
                .name("Vacuum state")
                .settable(VACUUM_NODE_STATE_PROP.settable)
                .retained(VACUUM_NODE_STATE_PROP.retained)
                .build(),
            )
            .add_property(
                VACUUM_NODE_ACTION_PROP.id,
                PropertyDescriptionBuilder::enumeration(config.actions.iter().map(|a| a.as_str()))
                    .unwrap()
                    .name("Vacuum action")
                    .settable(VACUUM_NODE_ACTION_PROP.settable)
                    .retained(VACUUM_NODE_ACTION_PROP.retained)
                    .build(),
            )
            .add_property_cond(
                VACUUM_NODE_SUCTION_PROP.id,
                !config.suction.is_empty(),
                || {
                    PropertyDescriptionBuilder::enumeration(
                        config.suction.iter().map(String::as_str),
                    )
                    .unwrap()
                    .name("Suction")
                    .settable(VACUUM_NODE_SUCTION_PROP.settable)
                    .retained(VACUUM_NODE_SUCTION_PROP.retained)
                    .build()
                },
            )
            .add_property_cond(VACUUM_NODE_ERROR_PROP.id, config.error, || {
                PropertyDescriptionBuilder::string()
                    .name("Error")
                    .settable(VACUUM_NODE_ERROR_PROP.settable)
                    .retained(VACUUM_NODE_ERROR_PROP.retained)
                    .build()
            });

        for consumable in &config.consumables {
            let spec = consumable.life_prop();
            db = db.add_property(
                spec.id,
                PropertyDescriptionBuilder::integer()
                    .name(consumable.life_name())
                    .integer_range(IntegerRange {
                        min: Some(0),
                        max: Some(100),
                        step: None,
                    })
                    .unit(HOMIE_UNIT_PERCENT)
                    .settable(spec.settable)
                    .retained(spec.retained)
                    .build(),
            );
        }

        db.add_property_cond(
            VACUUM_NODE_RESET_CONSUMABLE_PROP.id,
            !config.consumables.is_empty(),
            || {
                PropertyDescriptionBuilder::enumeration(
                    config.consumables.iter().map(|c| c.as_str()),
                )
                .unwrap()
                .name("Reset consumable")
                .settable(VACUUM_NODE_RESET_CONSUMABLE_PROP.settable)
                .retained(VACUUM_NODE_RESET_CONSUMABLE_PROP.retained)
                .build()
            },
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
    ) -> (HomieNodeDescription, VacuumNodePublisher) {
        (
            self.node_builder.build(),
            VacuumNodePublisher::new(
                NodeRef::new(
                    client.homie_domain().to_owned(),
                    client.id().clone(),
                    node_id,
                ),
                client.clone(),
                self.config,
            ),
        )
    }
}

// ── Publisher ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct VacuumNodePublisher {
    client: Homie5DeviceProtocol,
    config: VacuumNodeConfig,
    node: NodeRef,
}

impl VacuumNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol, config: VacuumNodeConfig) -> Self {
        Self {
            node,
            client,
            config,
        }
    }

    pub fn state(&self, state: VacuumState) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &VACUUM_NODE_STATE_PROP.id,
            state.as_str(),
            VACUUM_NODE_STATE_PROP.retained,
        )
    }

    /// `None` if the action is not configured.
    pub fn action(&self, action: VacuumAction) -> Option<homie5::client::Publish> {
        if !self.config.actions.contains(&action) {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &VACUUM_NODE_ACTION_PROP.id,
            action.as_str(),
            VACUUM_NODE_ACTION_PROP.retained,
        ))
    }

    /// `None` if `preset` is not one of the configured suction presets.
    pub fn suction(&self, preset: &str) -> Option<homie5::client::Publish> {
        if !self.config.suction.iter().any(|p| p == preset) {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &VACUUM_NODE_SUCTION_PROP.id,
            preset,
            VACUUM_NODE_SUCTION_PROP.retained,
        ))
    }

    /// `None` if `preset` is not one of the configured suction presets.
    pub fn suction_target(&self, preset: &str) -> Option<homie5::client::Publish> {
        if !self.config.suction.iter().any(|p| p == preset) {
            return None;
        }
        Some(self.client.publish_target(
            self.node.node_id(),
            &VACUUM_NODE_SUCTION_PROP.id,
            preset,
            VACUUM_NODE_SUCTION_PROP.retained,
        ))
    }

    /// Publishes the error text; an empty `text` means no error. `None` if
    /// `error` is not configured.
    pub fn error(&self, text: &str) -> Option<homie5::client::Publish> {
        if !self.config.error {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &VACUUM_NODE_ERROR_PROP.id,
            text,
            VACUUM_NODE_ERROR_PROP.retained,
        ))
    }

    /// Publishes the remaining life in percent, clamped to 0–100 (an overdue
    /// consumable reports 0). `None` if the consumable is not configured.
    pub fn consumable_life(
        &self,
        consumable: VacuumConsumable,
        percent: i64,
    ) -> Option<homie5::client::Publish> {
        if !self.config.consumables.contains(&consumable) {
            return None;
        }
        let spec = consumable.life_prop();
        Some(self.client.publish_value(
            self.node.node_id(),
            &spec.id,
            percent.clamp(0, 100).to_string(),
            spec.retained,
        ))
    }

    /// `None` if the consumable is not configured.
    pub fn reset_consumable(
        &self,
        consumable: VacuumConsumable,
    ) -> Option<homie5::client::Publish> {
        if !self.config.consumables.contains(&consumable) {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &VACUUM_NODE_RESET_CONSUMABLE_PROP.id,
            consumable.as_str(),
            VACUUM_NODE_RESET_CONSUMABLE_PROP.retained,
        ))
    }

    fn parse_enum(
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> Result<String, ParseError> {
        let property_id = property.prop_id().to_string();
        match desc.with_property(property, |prop_desc| {
            HomieValue::parse(set_value, prop_desc)
        }) {
            None => Err(ParseError::new(
                property_id,
                set_value,
                ParseErrorKind::MissingPropertyDescription,
            )),
            Some(Ok(HomieValue::Enum(value))) => Ok(value),
            Some(_) => Err(ParseError::new(
                property_id,
                set_value,
                ParseErrorKind::InvalidHomieValue,
            )),
        }
    }
}

impl SetCommandParser for VacuumNodePublisher {
    type Event = VacuumNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let invalid_variant = || {
            ParseOutcome::Invalid(ParseError::new(
                property.prop_id().to_string(),
                set_value,
                ParseErrorKind::InvalidVariant,
            ))
        };

        if property.match_with_node(&self.node, &VACUUM_NODE_ACTION_PROP.id) {
            match Self::parse_enum(property, desc, set_value) {
                Ok(value) => match value.parse() {
                    Ok(action) => ParseOutcome::Parsed(VacuumNodeSetEvents::Action(action)),
                    Err(_) => invalid_variant(),
                },
                Err(err) => ParseOutcome::Invalid(err),
            }
        } else if !self.config.suction.is_empty()
            && property.match_with_node(&self.node, &VACUUM_NODE_SUCTION_PROP.id)
        {
            match Self::parse_enum(property, desc, set_value) {
                Ok(value) => ParseOutcome::Parsed(VacuumNodeSetEvents::Suction(value)),
                Err(err) => ParseOutcome::Invalid(err),
            }
        } else if !self.config.consumables.is_empty()
            && property.match_with_node(&self.node, &VACUUM_NODE_RESET_CONSUMABLE_PROP.id)
        {
            match Self::parse_enum(property, desc, set_value) {
                Ok(value) => match value.parse() {
                    Ok(consumable) => {
                        ParseOutcome::Parsed(VacuumNodeSetEvents::ResetConsumable(consumable))
                    }
                    Err(_) => invalid_variant(),
                },
                Err(err) => ParseOutcome::Invalid(err),
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
                VACUUM_NODE_ACTION_PROP.id.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use homie5::{
        HomieDataType, HomieDomain,
        client::QoS,
        device_description::{DeviceDescriptionBuilder, HomiePropertyFormat},
    };

    fn full_config() -> VacuumNodeConfig {
        VacuumNodeConfig {
            actions: VacuumAction::ALL.to_vec(),
            suction: vec![
                "quiet".into(),
                "balanced".into(),
                "turbo".into(),
                "max".into(),
            ],
            error: true,
            consumables: vec![
                VacuumConsumable::MainBrush,
                VacuumConsumable::SideBrush,
                VacuumConsumable::Filter,
                VacuumConsumable::Sensor,
            ],
        }
    }

    fn enum_format(values: &[&str]) -> HomiePropertyFormat {
        HomiePropertyFormat::Enum(values.iter().map(|v| v.to_string()).collect())
    }

    fn fixture(
        config: &VacuumNodeConfig,
    ) -> (HomieDeviceDescription, VacuumNodePublisher, HomieID) {
        let device_id: HomieID = "olaf".try_into().unwrap();
        let (client, _) = Homie5DeviceProtocol::new(device_id.clone(), HomieDomain::Default);
        let (node_desc, publisher) = VacuumNodeBuilder::new(config)
            .unwrap()
            .build_with_publisher(VACUUM_NODE_DEFAULT_ID, &client);
        let desc = DeviceDescriptionBuilder::new()
            .name("vacuum test")
            .add_node(VACUUM_NODE_DEFAULT_ID, node_desc)
            .build();
        (desc, publisher, device_id)
    }

    fn prop_ref(device_id: &HomieID, node_id: &str, prop_id: HomieID) -> PropertyRef {
        PropertyRef::new(
            HomieDomain::Default,
            device_id.clone(),
            node_id.to_string().try_into().unwrap(),
            prop_id,
        )
    }

    // ── Descriptor shape ────────────────────────────────────────────────────

    #[test]
    fn empty_config_builds_state_and_action_only() {
        let config: VacuumNodeConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(config, VacuumNodeConfig::default());

        let node = VacuumNodeBuilder::new(&config).unwrap().build();
        assert_eq!(node.r#type.as_deref(), Some(SMARTHOME_CAP_VACUUM));
        assert_eq!(node.name.as_deref(), Some(VACUUM_NODE_DEFAULT_NAME));

        let mut ids: Vec<String> = node.properties.keys().map(|id| id.to_string()).collect();
        ids.sort();
        assert_eq!(ids, ["action", "state"]);

        let action = node.properties.get(&VACUUM_NODE_ACTION_PROP.id).unwrap();
        assert_eq!(
            action.format,
            enum_format(&["start", "pause", "stop", "dock", "spot", "locate"])
        );
    }

    #[test]
    fn state_is_a_retained_read_only_enum_of_all_states() {
        let node = VacuumNodeBuilder::new(&VacuumNodeConfig::default())
            .unwrap()
            .build();
        let state = node.properties.get(&VACUUM_NODE_STATE_PROP.id).unwrap();

        assert_eq!(state.datatype, HomieDataType::Enum);
        assert_eq!(
            state.format,
            enum_format(&["idle", "cleaning", "paused", "returning", "docked", "error"])
        );
        assert!(state.retained);
        assert!(!state.settable);
    }

    #[test]
    fn action_lists_the_configured_subset_and_is_settable_non_retained() {
        let config = VacuumNodeConfig {
            actions: vec![VacuumAction::Start, VacuumAction::Dock],
            ..VacuumNodeConfig::default()
        };
        let node = VacuumNodeBuilder::new(&config).unwrap().build();
        let action = node.properties.get(&VACUUM_NODE_ACTION_PROP.id).unwrap();

        assert_eq!(action.datatype, HomieDataType::Enum);
        assert_eq!(action.format, enum_format(&["start", "dock"]));
        assert!(!action.retained);
        assert!(action.settable);
    }

    #[test]
    fn full_config_describes_every_optional_property() {
        let node = VacuumNodeBuilder::new(&full_config()).unwrap().build();

        let suction = node.properties.get(&VACUUM_NODE_SUCTION_PROP.id).unwrap();
        assert_eq!(suction.datatype, HomieDataType::Enum);
        assert_eq!(
            suction.format,
            enum_format(&["quiet", "balanced", "turbo", "max"])
        );
        assert!(suction.retained);
        assert!(suction.settable);

        let error = node.properties.get(&VACUUM_NODE_ERROR_PROP.id).unwrap();
        assert_eq!(error.datatype, HomieDataType::String);
        assert!(error.retained);
        assert!(!error.settable);

        for consumable in full_config().consumables {
            let life = node
                .properties
                .get(&consumable.life_prop().id)
                .unwrap_or_else(|| panic!("{consumable}-life must exist"));
            assert_eq!(life.datatype, HomieDataType::Integer);
            assert_eq!(
                life.format,
                HomiePropertyFormat::IntegerRange(IntegerRange {
                    min: Some(0),
                    max: Some(100),
                    step: None,
                })
            );
            assert_eq!(life.unit.as_deref(), Some(HOMIE_UNIT_PERCENT));
            assert!(life.retained);
            assert!(!life.settable);
        }
        assert!(!node.properties.contains_key(&VACUUM_NODE_MOP_LIFE_PROP.id));

        let reset = node
            .properties
            .get(&VACUUM_NODE_RESET_CONSUMABLE_PROP.id)
            .unwrap();
        assert_eq!(reset.datatype, HomieDataType::Enum);
        assert_eq!(
            reset.format,
            enum_format(&["main-brush", "side-brush", "filter", "sensor"])
        );
        assert!(!reset.retained);
        assert!(reset.settable);

        assert_eq!(node.properties.len(), 2 + 2 + 4 + 1);
    }

    #[test]
    fn each_config_field_gates_only_its_own_properties() {
        let only_suction = VacuumNodeConfig {
            suction: vec!["eco".into()],
            ..VacuumNodeConfig::default()
        };
        let only_error = VacuumNodeConfig {
            error: true,
            ..VacuumNodeConfig::default()
        };
        let only_mop = VacuumNodeConfig {
            consumables: vec![VacuumConsumable::Mop],
            ..VacuumNodeConfig::default()
        };

        let cases = [
            (only_suction, vec!["action", "state", "suction"]),
            (only_error, vec!["action", "error", "state"]),
            (
                only_mop,
                vec!["action", "mop-life", "reset-consumable", "state"],
            ),
        ];
        for (config, expected) in cases {
            let node = VacuumNodeBuilder::new(&config).unwrap().build();
            let mut ids: Vec<String> = node.properties.keys().map(|id| id.to_string()).collect();
            ids.sort();
            assert_eq!(ids, expected, "config {config:?}");
        }
    }

    #[test]
    fn builder_rejects_invalid_enum_lists() {
        let no_actions = VacuumNodeConfig {
            actions: Vec::new(),
            ..VacuumNodeConfig::default()
        };
        assert_eq!(
            VacuumNodeBuilder::new(&no_actions).err(),
            Some(VacuumConfigError::InvalidActions(
                PropertyDescriptionValidationError::MissingEnumFormat
            ))
        );

        let duplicate_suction = VacuumNodeConfig {
            suction: vec!["max".into(), "max".into()],
            ..VacuumNodeConfig::default()
        };
        assert_eq!(
            VacuumNodeBuilder::new(&duplicate_suction).err(),
            Some(VacuumConfigError::InvalidSuction(
                PropertyDescriptionValidationError::DuplicateEnumValues
            ))
        );

        let empty_suction_name = VacuumNodeConfig {
            suction: vec![String::new()],
            ..VacuumNodeConfig::default()
        };
        assert_eq!(
            VacuumNodeBuilder::new(&empty_suction_name).err(),
            Some(VacuumConfigError::InvalidSuction(
                PropertyDescriptionValidationError::EnumContainsEmptyValue
            ))
        );

        let duplicate_consumable = VacuumNodeConfig {
            consumables: vec![VacuumConsumable::Filter, VacuumConsumable::Filter],
            ..VacuumNodeConfig::default()
        };
        assert_eq!(
            VacuumNodeBuilder::new(&duplicate_consumable).err(),
            Some(VacuumConfigError::InvalidConsumables(
                PropertyDescriptionValidationError::DuplicateEnumValues
            ))
        );
    }

    // ── Wire strings and config serde ───────────────────────────────────────

    #[test]
    fn enums_round_trip_through_their_wire_strings() {
        for state in VacuumState::ALL {
            assert_eq!(state.as_str().parse::<VacuumState>().unwrap(), state);
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, format!("\"{state}\""));
            assert_eq!(serde_json::from_str::<VacuumState>(&json).unwrap(), state);
        }
        for action in VacuumAction::ALL {
            assert_eq!(action.as_str().parse::<VacuumAction>().unwrap(), action);
            let json = serde_json::to_string(&action).unwrap();
            assert_eq!(serde_json::from_str::<VacuumAction>(&json).unwrap(), action);
        }
        for consumable in VacuumConsumable::ALL {
            assert_eq!(
                consumable.as_str().parse::<VacuumConsumable>().unwrap(),
                consumable
            );
            assert_eq!(
                consumable.life_prop().id.to_string(),
                format!("{consumable}-life")
            );
            let json = serde_json::to_string(&consumable).unwrap();
            assert_eq!(
                serde_json::from_str::<VacuumConsumable>(&json).unwrap(),
                consumable
            );
        }
        assert!("fly".parse::<VacuumAction>().is_err());
        assert!("MainBrush".parse::<VacuumConsumable>().is_err());
    }

    #[test]
    fn config_deserializes_wire_names() {
        let config: VacuumNodeConfig = serde_json::from_str(
            r#"{"actions":["start","dock"],"suction":["quiet","max"],"error":true,"consumables":["main-brush","mop"]}"#,
        )
        .unwrap();
        assert_eq!(
            config,
            VacuumNodeConfig {
                actions: vec![VacuumAction::Start, VacuumAction::Dock],
                suction: vec!["quiet".into(), "max".into()],
                error: true,
                consumables: vec![VacuumConsumable::MainBrush, VacuumConsumable::Mop],
            }
        );
        assert!(serde_json::from_str::<VacuumNodeConfig>(r#"{"actions":["fly"]}"#).is_err());
    }

    // ── Parsing ─────────────────────────────────────────────────────────────

    #[test]
    fn action_parses_valid_rejects_invalid_and_ignores_other_properties() {
        let (desc, publisher, device_id) = fixture(&full_config());
        let action = prop_ref(&device_id, "vacuum", VACUUM_NODE_ACTION_PROP.id);

        for expected in VacuumAction::ALL {
            assert_eq!(
                publisher.parse_set(&action, &desc, expected.as_str()),
                ParseOutcome::Parsed(VacuumNodeSetEvents::Action(expected))
            );
        }

        match publisher.parse_set(&action, &desc, "fly") {
            ParseOutcome::Invalid(err) => {
                assert_eq!(err.property_id, "action");
                assert_eq!(err.payload, "fly");
            }
            other => panic!("expected Invalid, got {other:?}"),
        }

        let state = prop_ref(&device_id, "vacuum", VACUUM_NODE_STATE_PROP.id);
        assert_eq!(
            publisher.parse_set(&state, &desc, "cleaning"),
            ParseOutcome::NoMatch
        );
        let other_node = prop_ref(&device_id, "switch", VACUUM_NODE_ACTION_PROP.id);
        assert_eq!(
            publisher.parse_set(&other_node, &desc, "start"),
            ParseOutcome::NoMatch
        );
    }

    #[test]
    fn unconfigured_action_is_invalid() {
        let config = VacuumNodeConfig {
            actions: vec![VacuumAction::Start, VacuumAction::Dock],
            ..VacuumNodeConfig::default()
        };
        let (desc, publisher, device_id) = fixture(&config);
        let action = prop_ref(&device_id, "vacuum", VACUUM_NODE_ACTION_PROP.id);

        assert!(matches!(
            publisher.parse_set(&action, &desc, "locate"),
            ParseOutcome::Invalid(_)
        ));
    }

    #[test]
    fn suction_parses_valid_rejects_invalid_and_ignores_other_properties() {
        let (desc, publisher, device_id) = fixture(&full_config());
        let suction = prop_ref(&device_id, "vacuum", VACUUM_NODE_SUCTION_PROP.id);

        assert_eq!(
            publisher.parse_set(&suction, &desc, "turbo"),
            ParseOutcome::Parsed(VacuumNodeSetEvents::Suction("turbo".into()))
        );
        assert!(matches!(
            publisher.parse_set(&suction, &desc, "ludicrous"),
            ParseOutcome::Invalid(_)
        ));
        assert!(matches!(
            publisher.parse_set(&suction, &desc, ""),
            ParseOutcome::Invalid(_)
        ));

        let error = prop_ref(&device_id, "vacuum", VACUUM_NODE_ERROR_PROP.id);
        assert_eq!(
            publisher.parse_set(&error, &desc, "stuck"),
            ParseOutcome::NoMatch
        );
        let other_node = prop_ref(&device_id, "other", VACUUM_NODE_SUCTION_PROP.id);
        assert_eq!(
            publisher.parse_set(&other_node, &desc, "turbo"),
            ParseOutcome::NoMatch
        );
    }

    #[test]
    fn reset_consumable_parses_valid_rejects_invalid_and_ignores_other_properties() {
        let (desc, publisher, device_id) = fixture(&full_config());
        let reset = prop_ref(&device_id, "vacuum", VACUUM_NODE_RESET_CONSUMABLE_PROP.id);

        assert_eq!(
            publisher.parse_set(&reset, &desc, "side-brush"),
            ParseOutcome::Parsed(VacuumNodeSetEvents::ResetConsumable(
                VacuumConsumable::SideBrush
            ))
        );
        // `mop` is a known consumable but not configured on this node.
        assert!(matches!(
            publisher.parse_set(&reset, &desc, "mop"),
            ParseOutcome::Invalid(_)
        ));
        assert!(matches!(
            publisher.parse_set(&reset, &desc, "battery"),
            ParseOutcome::Invalid(_)
        ));

        let life = prop_ref(&device_id, "vacuum", VACUUM_NODE_FILTER_LIFE_PROP.id);
        assert_eq!(
            publisher.parse_set(&life, &desc, "100"),
            ParseOutcome::NoMatch
        );
        let other_node = prop_ref(&device_id, "other", VACUUM_NODE_RESET_CONSUMABLE_PROP.id);
        assert_eq!(
            publisher.parse_set(&other_node, &desc, "filter"),
            ParseOutcome::NoMatch
        );
    }

    #[test]
    fn unconfigured_optional_properties_do_not_match() {
        let (desc, publisher, device_id) = fixture(&VacuumNodeConfig::default());

        let suction = prop_ref(&device_id, "vacuum", VACUUM_NODE_SUCTION_PROP.id);
        assert_eq!(
            publisher.parse_set(&suction, &desc, "turbo"),
            ParseOutcome::NoMatch
        );
        let reset = prop_ref(&device_id, "vacuum", VACUUM_NODE_RESET_CONSUMABLE_PROP.id);
        assert_eq!(
            publisher.parse_set(&reset, &desc, "filter"),
            ParseOutcome::NoMatch
        );
    }

    #[test]
    fn parse_set_event_rejects_non_set_messages() {
        let (desc, publisher, device_id) = fixture(&full_config());

        let set = Homie5Message::PropertySet {
            property: prop_ref(&device_id, "vacuum", VACUUM_NODE_ACTION_PROP.id),
            set_value: "dock".into(),
        };
        assert_eq!(
            publisher.parse_set_event(&desc, &set),
            ParseOutcome::Parsed(VacuumNodeSetEvents::Action(VacuumAction::Dock))
        );

        let not_set = Homie5Message::DeviceRemoval {
            device: homie5::DeviceRef::new(HomieDomain::Default, device_id),
        };
        match publisher.parse_set_event(&desc, &not_set) {
            ParseOutcome::Invalid(err) => {
                assert_eq!(err.kind, ParseErrorKind::UnexpectedMessageType)
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    // ── Publisher ───────────────────────────────────────────────────────────

    #[test]
    fn retained_properties_publish_retained_with_exactly_once() {
        let (_, publisher, _) = fixture(&full_config());

        let cases = [
            (
                publisher.state(VacuumState::Returning),
                "/vacuum/state",
                "returning",
            ),
            (publisher.suction("max").unwrap(), "/vacuum/suction", "max"),
            (
                publisher.error("Main brush stuck").unwrap(),
                "/vacuum/error",
                "Main brush stuck",
            ),
            // Homie 5 publishes an empty string as a single NUL byte.
            (publisher.error("").unwrap(), "/vacuum/error", "\0"),
            (
                publisher
                    .consumable_life(VacuumConsumable::MainBrush, 87)
                    .unwrap(),
                "/vacuum/main-brush-life",
                "87",
            ),
        ];
        for (publish, suffix, payload) in cases {
            assert_eq!(publish.topic, format!("homie/5/olaf{suffix}"));
            assert_eq!(publish.payload, payload.as_bytes());
            assert!(publish.retain, "{suffix} must be retained");
            assert_eq!(publish.qos, QoS::ExactlyOnce);
        }
    }

    #[test]
    fn suction_target_publishes_to_the_target_attribute() {
        let (_, publisher, _) = fixture(&full_config());

        let publish = publisher.suction_target("quiet").unwrap();
        assert_eq!(publish.topic, "homie/5/olaf/vacuum/suction/$target");
        assert_eq!(publish.payload, b"quiet");
        assert!(publish.retain);
        assert!(publisher.suction_target("ludicrous").is_none());
    }

    #[test]
    fn commands_publish_non_retained() {
        let (_, publisher, _) = fixture(&full_config());

        let action = publisher.action(VacuumAction::Locate).unwrap();
        assert_eq!(action.topic, "homie/5/olaf/vacuum/action");
        assert_eq!(action.payload, b"locate");
        assert!(!action.retain);
        assert_eq!(action.qos, QoS::AtMostOnce);

        let reset = publisher
            .reset_consumable(VacuumConsumable::Filter)
            .unwrap();
        assert_eq!(reset.topic, "homie/5/olaf/vacuum/reset-consumable");
        assert_eq!(reset.payload, b"filter");
        assert!(!reset.retain);
        assert_eq!(reset.qos, QoS::AtMostOnce);
    }

    #[test]
    fn consumable_life_is_clamped_to_a_percentage() {
        let (_, publisher, _) = fixture(&full_config());

        let overdue = publisher
            .consumable_life(VacuumConsumable::Sensor, -12)
            .unwrap();
        assert_eq!(overdue.payload, b"0");
        let fresh = publisher
            .consumable_life(VacuumConsumable::Filter, 140)
            .unwrap();
        assert_eq!(fresh.payload, b"100");
    }

    #[test]
    fn unconfigured_values_are_not_published() {
        let config = VacuumNodeConfig {
            actions: vec![VacuumAction::Start],
            ..VacuumNodeConfig::default()
        };
        let (_, publisher, _) = fixture(&config);

        assert!(publisher.action(VacuumAction::Start).is_some());
        assert!(publisher.action(VacuumAction::Spot).is_none());
        assert!(publisher.suction("max").is_none());
        assert!(publisher.error("").is_none());
        assert!(
            publisher
                .consumable_life(VacuumConsumable::Filter, 50)
                .is_none()
        );
        assert!(
            publisher
                .reset_consumable(VacuumConsumable::Filter)
                .is_none()
        );

        let (_, full, _) = fixture(&full_config());
        assert!(full.suction("ludicrous").is_none());
        assert!(full.consumable_life(VacuumConsumable::Mop, 50).is_none());
    }
}

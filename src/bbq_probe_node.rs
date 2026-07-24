use homie5::{
    HOMIE_UNIT_DEGREE_CELSIUS, Homie5DeviceProtocol, Homie5Message, HomieColorValue, HomieID,
    HomieValue, NodeRef, PropertyRef,
    device_description::{
        ColorFormat, FloatRange, HomieDeviceDescription, HomieNodeDescription,
        NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::{ParseError, ParseErrorKind, ParseOutcome, SMARTHOME_CAP_BBQ_PROBE, SetCommandParser};

pub const BBQ_PROBE_NODE_DEFAULT_ID: HomieID = HomieID::new_const("bbq-probe");
pub const BBQ_PROBE_NODE_DEFAULT_NAME: &str = "BBQ Probe";
pub const BBQ_PROBE_NODE_TEMPERATURE_PROP_ID: HomieID = HomieID::new_const("temperature");
pub const BBQ_PROBE_NODE_CONNECTED_PROP_ID: HomieID = HomieID::new_const("connected");
pub const BBQ_PROBE_NODE_LOW_THRESHOLD_PROP_ID: HomieID = HomieID::new_const("low-threshold");
pub const BBQ_PROBE_NODE_HIGH_THRESHOLD_PROP_ID: HomieID = HomieID::new_const("high-threshold");
pub const BBQ_PROBE_NODE_THRESHOLD_STATE_PROP_ID: HomieID = HomieID::new_const("threshold-state");
pub const BBQ_PROBE_NODE_ROLE_PROP_ID: HomieID = HomieID::new_const("role");
pub const BBQ_PROBE_NODE_LABEL_PROP_ID: HomieID = HomieID::new_const("label");
pub const BBQ_PROBE_NODE_ALARM_MODE_PROP_ID: HomieID = HomieID::new_const("alarm-mode");
pub const BBQ_PROBE_NODE_COLOR_PROP_ID: HomieID = HomieID::new_const("color");
pub const BBQ_PROBE_NODE_SENSOR_TYPE_PROP_ID: HomieID = HomieID::new_const("sensor-type");

pub const BBQ_PROBE_ROLE_PIT: &str = "pit";
pub const BBQ_PROBE_ROLE_FOOD: &str = "food";
pub const BBQ_PROBE_ROLE_AMBIENT: &str = "ambient";
pub const BBQ_PROBE_ROLE_OTHER: &str = "other";

// ── Threshold state ─────────────────────────────────────────────────────────

/// Derived threshold state of a probe.
///
/// Threshold comparisons are inclusive: `temperature <= low-threshold` is
/// `Low` and `temperature >= high-threshold` is `High`. A disconnected probe
/// is always `Unavailable`.
#[derive(Debug, Default, Copy, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum BbqProbeThresholdState {
    Low,
    #[default]
    Normal,
    High,
    Unavailable,
}

impl BbqProbeThresholdState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            BbqProbeThresholdState::Low => "low",
            BbqProbeThresholdState::Normal => "normal",
            BbqProbeThresholdState::High => "high",
            BbqProbeThresholdState::Unavailable => "unavailable",
        }
    }

    pub const fn variants() -> [&'static str; 4] {
        ["low", "normal", "high", "unavailable"]
    }

    /// Derive the threshold state from the current probe reading.
    ///
    /// Returns `None` when both thresholds would match at the same time
    /// (invalid configuration, e.g. `low >= high` with a temperature between
    /// them). Producers should report a configuration error in that case
    /// instead of silently choosing a state.
    pub fn derive(
        connected: bool,
        temperature: f64,
        low_threshold: Option<f64>,
        high_threshold: Option<f64>,
    ) -> Option<Self> {
        if !connected {
            return Some(BbqProbeThresholdState::Unavailable);
        }
        let low = low_threshold.is_some_and(|low| temperature <= low);
        let high = high_threshold.is_some_and(|high| temperature >= high);
        match (low, high) {
            (true, true) => None,
            (true, false) => Some(BbqProbeThresholdState::Low),
            (false, true) => Some(BbqProbeThresholdState::High),
            (false, false) => Some(BbqProbeThresholdState::Normal),
        }
    }
}

impl From<&BbqProbeThresholdState> for &'static str {
    fn from(value: &BbqProbeThresholdState) -> Self {
        value.as_str()
    }
}

impl From<&BbqProbeThresholdState> for String {
    fn from(value: &BbqProbeThresholdState) -> Self {
        value.as_str().to_string()
    }
}

impl TryFrom<&str> for BbqProbeThresholdState {
    type Error = homie5::Homie5ProtocolError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "low" => Ok(BbqProbeThresholdState::Low),
            "normal" => Ok(BbqProbeThresholdState::Normal),
            "high" => Ok(BbqProbeThresholdState::High),
            "unavailable" => Ok(BbqProbeThresholdState::Unavailable),
            _ => Err(homie5::Homie5ProtocolError::InvalidPayload),
        }
    }
}

// ── Node (state) ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct BbqProbeNode {
    pub publisher: BbqProbeNodePublisher,
    /// Last valid temperature reading. `None` when the probe was never
    /// connected. A disconnected probe keeps its last valid reading.
    pub temperature: Option<f64>,
    pub connected: bool,
    pub low_threshold: Option<f64>,
    pub low_threshold_target: Option<f64>,
    pub high_threshold: Option<f64>,
    pub high_threshold_target: Option<f64>,
    pub threshold_state: Option<BbqProbeThresholdState>,
    pub role: Option<String>,
    pub label: Option<String>,
    pub alarm_mode: Option<String>,
    pub color: Option<HomieColorValue>,
    pub sensor_type: Option<String>,
}

// ── Set events ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum BbqProbeNodeSetEvents {
    LowThreshold(f64),
    HighThreshold(f64),
    Role(String),
    Label(String),
    AlarmMode(String),
    Color(HomieColorValue),
    SensorType(String),
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BbqProbeNodeConfig {
    /// Temperature unit shared by `temperature` and both thresholds.
    pub unit: String,
    /// Numeric range shared by `temperature` and both thresholds.
    pub temp_range: FloatRange,
    /// Include `low-threshold` and `high-threshold`.
    pub thresholds: bool,
    pub thresholds_settable: bool,
    /// Include the derived `threshold-state` property.
    pub threshold_state: bool,
    /// Include the `role` property.
    pub role: bool,
    pub role_settable: bool,
    /// Allowed role variants. Defaults to `pit`, `food`, `ambient`, `other`.
    pub roles: Vec<String>,
    /// Include the `label` property.
    pub label: bool,
    pub label_settable: bool,
    /// Include the `alarm-mode` property. Requires `alarm_modes` variants.
    pub alarm_mode: bool,
    pub alarm_mode_settable: bool,
    /// Allowed alarm mode variants (producer-defined, no generic default).
    pub alarm_modes: Vec<String>,
    /// Include the `color` property (Homie color datatype, `rgb` format).
    pub color: bool,
    pub color_settable: bool,
    /// Include the `sensor-type` property. Requires `sensor_types` variants.
    pub sensor_type: bool,
    pub sensor_type_settable: bool,
    /// Allowed sensor type variants (producer-defined, no generic default).
    pub sensor_types: Vec<String>,
}

impl Default for BbqProbeNodeConfig {
    fn default() -> Self {
        Self {
            unit: HOMIE_UNIT_DEGREE_CELSIUS.to_string(),
            temp_range: FloatRange {
                min: Some(-40.0),
                max: Some(400.0),
                step: None,
            },
            thresholds: true,
            thresholds_settable: true,
            threshold_state: true,
            role: true,
            role_settable: true,
            roles: vec![
                BBQ_PROBE_ROLE_PIT.to_string(),
                BBQ_PROBE_ROLE_FOOD.to_string(),
                BBQ_PROBE_ROLE_AMBIENT.to_string(),
                BBQ_PROBE_ROLE_OTHER.to_string(),
            ],
            label: true,
            label_settable: true,
            alarm_mode: false,
            alarm_mode_settable: true,
            alarm_modes: vec![],
            color: false,
            color_settable: true,
            sensor_type: false,
            sensor_type_settable: true,
            sensor_types: vec![],
        }
    }
}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct BbqProbeNodeBuilder {
    config: BbqProbeNodeConfig,
    node_builder: NodeDescriptionBuilder,
}

impl BbqProbeNodeBuilder {
    pub fn new(config: &BbqProbeNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(BBQ_PROBE_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_BBQ_PROBE);

        Self {
            node_builder: db,
            config: config.clone(),
        }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &BbqProbeNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            BBQ_PROBE_NODE_TEMPERATURE_PROP_ID,
            PropertyDescriptionBuilder::float()
                .name("Probe temperature")
                .float_range(config.temp_range.clone())
                .unit(config.unit.to_owned())
                .settable(false)
                .retained(true)
                .build(),
        )
        .add_property(
            BBQ_PROBE_NODE_CONNECTED_PROP_ID,
            PropertyDescriptionBuilder::boolean()
                .name("Probe connected")
                .boolean_labels("disconnected", "connected")
                .settable(false)
                .retained(true)
                .build(),
        )
        .add_property_cond(
            BBQ_PROBE_NODE_LOW_THRESHOLD_PROP_ID,
            config.thresholds,
            || {
                PropertyDescriptionBuilder::float()
                    .name("Low temperature threshold")
                    .float_range(config.temp_range.clone())
                    .unit(config.unit.to_owned())
                    .settable(config.thresholds_settable)
                    .retained(true)
                    .build()
            },
        )
        .add_property_cond(
            BBQ_PROBE_NODE_HIGH_THRESHOLD_PROP_ID,
            config.thresholds,
            || {
                PropertyDescriptionBuilder::float()
                    .name("High temperature threshold")
                    .float_range(config.temp_range.clone())
                    .unit(config.unit.to_owned())
                    .settable(config.thresholds_settable)
                    .retained(true)
                    .build()
            },
        )
        .add_property_cond(
            BBQ_PROBE_NODE_THRESHOLD_STATE_PROP_ID,
            config.threshold_state,
            || {
                PropertyDescriptionBuilder::enumeration(BbqProbeThresholdState::variants())
                    .unwrap()
                    .name("Threshold state")
                    .settable(false)
                    .retained(true)
                    .build()
            },
        )
        .add_property_cond(BBQ_PROBE_NODE_ROLE_PROP_ID, config.role, || {
            PropertyDescriptionBuilder::enumeration(config.roles.iter().map(String::as_str))
                .unwrap()
                .name("Probe role")
                .settable(config.role_settable)
                .retained(true)
                .build()
        })
        .add_property_cond(BBQ_PROBE_NODE_LABEL_PROP_ID, config.label, || {
            PropertyDescriptionBuilder::string()
                .name("Probe label")
                .settable(config.label_settable)
                .retained(true)
                .build()
        })
        .add_property_cond(BBQ_PROBE_NODE_ALARM_MODE_PROP_ID, config.alarm_mode, || {
            PropertyDescriptionBuilder::enumeration(config.alarm_modes.iter().map(String::as_str))
                .unwrap()
                .name("Alarm mode")
                .settable(config.alarm_mode_settable)
                .retained(true)
                .build()
        })
        .add_property_cond(BBQ_PROBE_NODE_COLOR_PROP_ID, config.color, || {
            PropertyDescriptionBuilder::color(vec![ColorFormat::Rgb])
                .unwrap()
                .name("Probe color")
                .settable(config.color_settable)
                .retained(true)
                .build()
        })
        .add_property_cond(
            BBQ_PROBE_NODE_SENSOR_TYPE_PROP_ID,
            config.sensor_type,
            || {
                PropertyDescriptionBuilder::enumeration(
                    config.sensor_types.iter().map(String::as_str),
                )
                .unwrap()
                .name("Sensor type")
                .settable(config.sensor_type_settable)
                .retained(true)
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
    ) -> (HomieNodeDescription, BbqProbeNodePublisher) {
        (
            self.node_builder.build(),
            BbqProbeNodePublisher::new(
                NodeRef::new(
                    client.homie_domain().to_owned(),
                    client.id().to_owned(),
                    node_id,
                ),
                client.clone(),
                self.config,
            ),
        )
    }
}

// ── Publisher ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct BbqProbeNodePublisher {
    client: Homie5DeviceProtocol,
    config: BbqProbeNodeConfig,
    node: NodeRef,
    temperature_prop: HomieID,
    connected_prop: HomieID,
    low_threshold_prop: HomieID,
    high_threshold_prop: HomieID,
    threshold_state_prop: HomieID,
    role_prop: HomieID,
    label_prop: HomieID,
    alarm_mode_prop: HomieID,
    color_prop: HomieID,
    sensor_type_prop: HomieID,
}

impl BbqProbeNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol, config: BbqProbeNodeConfig) -> Self {
        Self {
            node,
            client,
            config,
            temperature_prop: BBQ_PROBE_NODE_TEMPERATURE_PROP_ID,
            connected_prop: BBQ_PROBE_NODE_CONNECTED_PROP_ID,
            low_threshold_prop: BBQ_PROBE_NODE_LOW_THRESHOLD_PROP_ID,
            high_threshold_prop: BBQ_PROBE_NODE_HIGH_THRESHOLD_PROP_ID,
            threshold_state_prop: BBQ_PROBE_NODE_THRESHOLD_STATE_PROP_ID,
            role_prop: BBQ_PROBE_NODE_ROLE_PROP_ID,
            label_prop: BBQ_PROBE_NODE_LABEL_PROP_ID,
            alarm_mode_prop: BBQ_PROBE_NODE_ALARM_MODE_PROP_ID,
            color_prop: BBQ_PROBE_NODE_COLOR_PROP_ID,
            sensor_type_prop: BBQ_PROBE_NODE_SENSOR_TYPE_PROP_ID,
        }
    }

    pub fn temperature(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.temperature_prop,
            value.to_string(),
            true,
        )
    }

    pub fn connected(&self, value: bool) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.connected_prop,
            value.to_string(),
            true,
        )
    }

    pub fn low_threshold(&self, value: f64) -> Option<homie5::client::Publish> {
        if !self.config.thresholds {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &self.low_threshold_prop,
            value.to_string(),
            true,
        ))
    }

    pub fn low_threshold_target(&self, value: f64) -> Option<homie5::client::Publish> {
        if !self.config.thresholds {
            return None;
        }
        Some(self.client.publish_target(
            self.node.node_id(),
            &self.low_threshold_prop,
            value.to_string(),
            true,
        ))
    }

    pub fn high_threshold(&self, value: f64) -> Option<homie5::client::Publish> {
        if !self.config.thresholds {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &self.high_threshold_prop,
            value.to_string(),
            true,
        ))
    }

    pub fn high_threshold_target(&self, value: f64) -> Option<homie5::client::Publish> {
        if !self.config.thresholds {
            return None;
        }
        Some(self.client.publish_target(
            self.node.node_id(),
            &self.high_threshold_prop,
            value.to_string(),
            true,
        ))
    }

    pub fn threshold_state(
        &self,
        value: BbqProbeThresholdState,
    ) -> Option<homie5::client::Publish> {
        if !self.config.threshold_state {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &self.threshold_state_prop,
            value.as_str(),
            true,
        ))
    }

    pub fn role(&self, value: &str) -> Option<homie5::client::Publish> {
        if !self.config.role {
            return None;
        }
        Some(
            self.client
                .publish_value(self.node.node_id(), &self.role_prop, value, true),
        )
    }

    pub fn role_target(&self, value: &str) -> Option<homie5::client::Publish> {
        if !self.config.role {
            return None;
        }
        Some(
            self.client
                .publish_target(self.node.node_id(), &self.role_prop, value, true),
        )
    }

    pub fn label(&self, value: &str) -> Option<homie5::client::Publish> {
        if !self.config.label {
            return None;
        }
        Some(
            self.client
                .publish_value(self.node.node_id(), &self.label_prop, value, true),
        )
    }

    pub fn label_target(&self, value: &str) -> Option<homie5::client::Publish> {
        if !self.config.label {
            return None;
        }
        Some(
            self.client
                .publish_target(self.node.node_id(), &self.label_prop, value, true),
        )
    }

    pub fn alarm_mode(&self, value: &str) -> Option<homie5::client::Publish> {
        if !self.config.alarm_mode {
            return None;
        }
        Some(
            self.client
                .publish_value(self.node.node_id(), &self.alarm_mode_prop, value, true),
        )
    }

    pub fn alarm_mode_target(&self, value: &str) -> Option<homie5::client::Publish> {
        if !self.config.alarm_mode {
            return None;
        }
        Some(
            self.client
                .publish_target(self.node.node_id(), &self.alarm_mode_prop, value, true),
        )
    }

    pub fn color(&self, value: HomieColorValue) -> Option<homie5::client::Publish> {
        if !self.config.color {
            return None;
        }
        Some(
            self.client
                .publish_value(self.node.node_id(), &self.color_prop, value, true),
        )
    }

    pub fn color_target(&self, value: HomieColorValue) -> Option<homie5::client::Publish> {
        if !self.config.color {
            return None;
        }
        Some(
            self.client
                .publish_target(self.node.node_id(), &self.color_prop, value, true),
        )
    }

    pub fn sensor_type(&self, value: &str) -> Option<homie5::client::Publish> {
        if !self.config.sensor_type {
            return None;
        }
        Some(
            self.client
                .publish_value(self.node.node_id(), &self.sensor_type_prop, value, true),
        )
    }

    pub fn sensor_type_target(&self, value: &str) -> Option<homie5::client::Publish> {
        if !self.config.sensor_type {
            return None;
        }
        Some(
            self.client
                .publish_target(self.node.node_id(), &self.sensor_type_prop, value, true),
        )
    }
}

impl SetCommandParser for BbqProbeNodePublisher {
    type Event = BbqProbeNodeSetEvents;

    fn parse_set(
        &self,
        property: &PropertyRef,
        desc: &HomieDeviceDescription,
        set_value: &str,
    ) -> ParseOutcome<Self::Event> {
        let property_id = property.prop_id().to_string();

        if property.match_with_node(&self.node, &self.low_threshold_prop) {
            if !self.config.thresholds || !self.config.thresholds_settable {
                return ParseOutcome::NoMatch;
            }
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
                    ParseOutcome::Parsed(BbqProbeNodeSetEvents::LowThreshold(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property_id,
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &self.high_threshold_prop) {
            if !self.config.thresholds || !self.config.thresholds_settable {
                return ParseOutcome::NoMatch;
            }
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
                    ParseOutcome::Parsed(BbqProbeNodeSetEvents::HighThreshold(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property_id,
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &self.role_prop) {
            if !self.config.role || !self.config.role_settable {
                return ParseOutcome::NoMatch;
            }
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
                Ok(HomieValue::Enum(value)) => {
                    ParseOutcome::Parsed(BbqProbeNodeSetEvents::Role(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property_id,
                    set_value,
                    ParseErrorKind::InvalidVariant,
                )),
            }
        } else if property.match_with_node(&self.node, &self.label_prop) {
            if !self.config.label || !self.config.label_settable {
                return ParseOutcome::NoMatch;
            }
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
                    ParseOutcome::Parsed(BbqProbeNodeSetEvents::Label(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property_id,
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &self.alarm_mode_prop) {
            if !self.config.alarm_mode || !self.config.alarm_mode_settable {
                return ParseOutcome::NoMatch;
            }
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
                Ok(HomieValue::Enum(value)) => {
                    ParseOutcome::Parsed(BbqProbeNodeSetEvents::AlarmMode(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property_id,
                    set_value,
                    ParseErrorKind::InvalidVariant,
                )),
            }
        } else if property.match_with_node(&self.node, &self.color_prop) {
            if !self.config.color || !self.config.color_settable {
                return ParseOutcome::NoMatch;
            }
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
                    ParseOutcome::Parsed(BbqProbeNodeSetEvents::Color(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property_id,
                    set_value,
                    ParseErrorKind::InvalidHomieValue,
                )),
            }
        } else if property.match_with_node(&self.node, &self.sensor_type_prop) {
            if !self.config.sensor_type || !self.config.sensor_type_settable {
                return ParseOutcome::NoMatch;
            }
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
                Ok(HomieValue::Enum(value)) => {
                    ParseOutcome::Parsed(BbqProbeNodeSetEvents::SensorType(value))
                }
                _ => ParseOutcome::Invalid(ParseError::new(
                    property_id,
                    set_value,
                    ParseErrorKind::InvalidVariant,
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
                self.temperature_prop.to_string(),
                "",
                ParseErrorKind::UnexpectedMessageType,
            )),
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use homie5::{
        HomieDomain,
        device_description::{DeviceDescriptionBuilder, HomiePropertyFormat},
    };

    fn base_disabled_config() -> BbqProbeNodeConfig {
        BbqProbeNodeConfig {
            unit: "°F".to_string(),
            temp_range: FloatRange {
                min: Some(0.0),
                max: Some(750.0),
                step: None,
            },
            thresholds: false,
            thresholds_settable: true,
            threshold_state: false,
            role: false,
            role_settable: true,
            roles: vec!["pit".to_string(), "food".to_string()],
            label: false,
            label_settable: true,
            alarm_mode: false,
            alarm_mode_settable: true,
            alarm_modes: vec!["off".to_string(), "push".to_string()],
            color: false,
            color_settable: true,
            sensor_type: false,
            sensor_type_settable: true,
            sensor_types: vec!["1000K/Maverick".to_string(), "iGrill2".to_string()],
        }
    }

    fn full_config() -> BbqProbeNodeConfig {
        BbqProbeNodeConfig {
            alarm_mode: true,
            alarm_modes: vec![
                "off".to_string(),
                "push".to_string(),
                "buzzer".to_string(),
                "push-buzzer".to_string(),
            ],
            color: true,
            sensor_type: true,
            sensor_types: vec!["1000K/Maverick".to_string(), "iGrill2".to_string()],
            ..BbqProbeNodeConfig::default()
        }
    }

    #[test]
    fn mandatory_properties_have_expected_shape() {
        let config = base_disabled_config();
        let node = BbqProbeNodeBuilder::new(&config).build();

        assert_eq!(node.r#type.as_deref(), Some(SMARTHOME_CAP_BBQ_PROBE));

        let temperature = node
            .properties
            .get(&BBQ_PROBE_NODE_TEMPERATURE_PROP_ID)
            .expect("temperature property must exist");
        assert_eq!(temperature.unit.as_deref(), Some("°F"));
        assert!(!temperature.settable);
        assert!(temperature.retained);
        assert_eq!(
            temperature.format,
            HomiePropertyFormat::FloatRange(FloatRange {
                min: Some(0.0),
                max: Some(750.0),
                step: None,
            })
        );

        let connected = node
            .properties
            .get(&BBQ_PROBE_NODE_CONNECTED_PROP_ID)
            .expect("connected property must exist");
        assert!(!connected.settable);
        assert!(connected.retained);

        // With everything disabled only the two mandatory properties exist.
        assert_eq!(node.properties.len(), 2);
    }

    #[test]
    fn each_config_flag_gates_the_expected_optional_properties() {
        struct Case {
            enable: fn(&mut BbqProbeNodeConfig),
            expected: Vec<HomieID>,
        }
        let cases = [
            Case {
                enable: |c| c.thresholds = true,
                expected: vec![
                    BBQ_PROBE_NODE_LOW_THRESHOLD_PROP_ID,
                    BBQ_PROBE_NODE_HIGH_THRESHOLD_PROP_ID,
                ],
            },
            Case {
                enable: |c| c.threshold_state = true,
                expected: vec![BBQ_PROBE_NODE_THRESHOLD_STATE_PROP_ID],
            },
            Case {
                enable: |c| c.role = true,
                expected: vec![BBQ_PROBE_NODE_ROLE_PROP_ID],
            },
            Case {
                enable: |c| c.label = true,
                expected: vec![BBQ_PROBE_NODE_LABEL_PROP_ID],
            },
            Case {
                enable: |c| c.alarm_mode = true,
                expected: vec![BBQ_PROBE_NODE_ALARM_MODE_PROP_ID],
            },
            Case {
                enable: |c| c.color = true,
                expected: vec![BBQ_PROBE_NODE_COLOR_PROP_ID],
            },
            Case {
                enable: |c| c.sensor_type = true,
                expected: vec![BBQ_PROBE_NODE_SENSOR_TYPE_PROP_ID],
            },
        ];

        let all_optional = [
            BBQ_PROBE_NODE_LOW_THRESHOLD_PROP_ID,
            BBQ_PROBE_NODE_HIGH_THRESHOLD_PROP_ID,
            BBQ_PROBE_NODE_THRESHOLD_STATE_PROP_ID,
            BBQ_PROBE_NODE_ROLE_PROP_ID,
            BBQ_PROBE_NODE_LABEL_PROP_ID,
            BBQ_PROBE_NODE_ALARM_MODE_PROP_ID,
            BBQ_PROBE_NODE_COLOR_PROP_ID,
            BBQ_PROBE_NODE_SENSOR_TYPE_PROP_ID,
        ];

        for case in &cases {
            let mut config = base_disabled_config();
            (case.enable)(&mut config);
            let node = BbqProbeNodeBuilder::new(&config).build();

            for prop_id in &all_optional {
                let expected_present = case.expected.contains(prop_id);
                assert_eq!(
                    node.properties.contains_key(prop_id),
                    expected_present,
                    "property {prop_id} presence mismatch"
                );
            }
        }
    }

    #[test]
    fn settability_flags_are_respected_in_description() {
        let mut config = full_config();
        config.thresholds_settable = false;
        config.role_settable = false;
        config.label_settable = false;
        config.alarm_mode_settable = false;
        config.color_settable = false;
        config.sensor_type_settable = false;

        let node = BbqProbeNodeBuilder::new(&config).build();
        for prop_id in [
            BBQ_PROBE_NODE_LOW_THRESHOLD_PROP_ID,
            BBQ_PROBE_NODE_HIGH_THRESHOLD_PROP_ID,
            BBQ_PROBE_NODE_ROLE_PROP_ID,
            BBQ_PROBE_NODE_LABEL_PROP_ID,
            BBQ_PROBE_NODE_ALARM_MODE_PROP_ID,
            BBQ_PROBE_NODE_COLOR_PROP_ID,
            BBQ_PROBE_NODE_SENSOR_TYPE_PROP_ID,
        ] {
            let prop = node
                .properties
                .get(&prop_id)
                .expect("property must be present");
            assert!(!prop.settable, "property {prop_id} must not be settable");
        }
    }

    #[test]
    fn enum_properties_use_configured_variants() {
        let node = BbqProbeNodeBuilder::new(&full_config()).build();

        let role = node.properties.get(&BBQ_PROBE_NODE_ROLE_PROP_ID).unwrap();
        assert_eq!(
            role.format,
            HomiePropertyFormat::Enum(vec![
                "pit".to_string(),
                "food".to_string(),
                "ambient".to_string(),
                "other".to_string(),
            ])
        );

        let alarm_mode = node
            .properties
            .get(&BBQ_PROBE_NODE_ALARM_MODE_PROP_ID)
            .unwrap();
        assert_eq!(
            alarm_mode.format,
            HomiePropertyFormat::Enum(vec![
                "off".to_string(),
                "push".to_string(),
                "buzzer".to_string(),
                "push-buzzer".to_string(),
            ])
        );

        let sensor_type = node
            .properties
            .get(&BBQ_PROBE_NODE_SENSOR_TYPE_PROP_ID)
            .unwrap();
        assert_eq!(
            sensor_type.format,
            HomiePropertyFormat::Enum(vec!["1000K/Maverick".to_string(), "iGrill2".to_string(),])
        );

        let threshold_state = node
            .properties
            .get(&BBQ_PROBE_NODE_THRESHOLD_STATE_PROP_ID)
            .unwrap();
        assert_eq!(
            threshold_state.format,
            HomiePropertyFormat::Enum(vec![
                "low".to_string(),
                "normal".to_string(),
                "high".to_string(),
                "unavailable".to_string(),
            ])
        );
    }

    #[test]
    fn threshold_state_derivation_is_inclusive() {
        use BbqProbeThresholdState as S;

        assert_eq!(
            S::derive(false, 100.0, Some(50.0), Some(120.0)),
            Some(S::Unavailable)
        );
        assert_eq!(
            S::derive(true, 50.0, Some(50.0), Some(120.0)),
            Some(S::Low),
            "temperature == low threshold must be low (inclusive)"
        );
        assert_eq!(
            S::derive(true, 120.0, Some(50.0), Some(120.0)),
            Some(S::High),
            "temperature == high threshold must be high (inclusive)"
        );
        assert_eq!(
            S::derive(true, 80.0, Some(50.0), Some(120.0)),
            Some(S::Normal)
        );
        assert_eq!(S::derive(true, 80.0, None, None), Some(S::Normal));
        assert_eq!(S::derive(true, 40.0, Some(50.0), None), Some(S::Low));
        assert_eq!(S::derive(true, 130.0, None, Some(120.0)), Some(S::High));
        assert_eq!(
            S::derive(true, 80.0, Some(100.0), Some(60.0)),
            None,
            "conflicting thresholds must be reported, not silently resolved"
        );
    }

    fn publisher_fixture() -> (
        HomieDeviceDescription,
        BbqProbeNodePublisher,
        Homie5DeviceProtocol,
        HomieID,
    ) {
        let device_id: HomieID = "bbq-test-device".try_into().unwrap();
        let (client, _) = Homie5DeviceProtocol::new(device_id.clone(), HomieDomain::Default);
        let node_id: HomieID = "probe-1".try_into().unwrap();
        let (node_desc, publisher) =
            BbqProbeNodeBuilder::new(&full_config()).build_with_publisher(node_id.clone(), &client);
        let desc = DeviceDescriptionBuilder::new()
            .name("bbq test")
            .add_node(node_id, node_desc)
            .build();
        (desc, publisher, client, device_id)
    }

    fn prop_ref(device_id: &HomieID, node_id: &str, prop_id: HomieID) -> PropertyRef {
        PropertyRef::new(
            HomieDomain::Default,
            device_id.clone(),
            node_id.to_string().try_into().unwrap(),
            prop_id,
        )
    }

    #[test]
    fn parse_set_handles_valid_invalid_and_unrelated_commands() {
        let (desc, publisher, _client, device_id) = publisher_fixture();

        // Valid float threshold.
        let low = prop_ref(&device_id, "probe-1", BBQ_PROBE_NODE_LOW_THRESHOLD_PROP_ID);
        match publisher.parse_set(&low, &desc, "55.5") {
            ParseOutcome::Parsed(BbqProbeNodeSetEvents::LowThreshold(v)) => assert_eq!(v, 55.5),
            other => panic!("expected parsed low threshold, got {other:?}"),
        }

        // Out-of-range threshold is invalid.
        let high = prop_ref(&device_id, "probe-1", BBQ_PROBE_NODE_HIGH_THRESHOLD_PROP_ID);
        assert!(matches!(
            publisher.parse_set(&high, &desc, "9999"),
            ParseOutcome::Invalid(_)
        ));

        // Valid enum variants.
        let role = prop_ref(&device_id, "probe-1", BBQ_PROBE_NODE_ROLE_PROP_ID);
        match publisher.parse_set(&role, &desc, "food") {
            ParseOutcome::Parsed(BbqProbeNodeSetEvents::Role(v)) => assert_eq!(v, "food"),
            other => panic!("expected parsed role, got {other:?}"),
        }
        assert!(matches!(
            publisher.parse_set(&role, &desc, "not-a-role"),
            ParseOutcome::Invalid(_)
        ));

        let alarm = prop_ref(&device_id, "probe-1", BBQ_PROBE_NODE_ALARM_MODE_PROP_ID);
        match publisher.parse_set(&alarm, &desc, "push-buzzer") {
            ParseOutcome::Parsed(BbqProbeNodeSetEvents::AlarmMode(v)) => {
                assert_eq!(v, "push-buzzer")
            }
            other => panic!("expected parsed alarm mode, got {other:?}"),
        }

        let sensor = prop_ref(&device_id, "probe-1", BBQ_PROBE_NODE_SENSOR_TYPE_PROP_ID);
        match publisher.parse_set(&sensor, &desc, "iGrill2") {
            ParseOutcome::Parsed(BbqProbeNodeSetEvents::SensorType(v)) => {
                assert_eq!(v, "iGrill2")
            }
            other => panic!("expected parsed sensor type, got {other:?}"),
        }

        // Valid label.
        let label = prop_ref(&device_id, "probe-1", BBQ_PROBE_NODE_LABEL_PROP_ID);
        match publisher.parse_set(&label, &desc, "Brisket") {
            ParseOutcome::Parsed(BbqProbeNodeSetEvents::Label(v)) => assert_eq!(v, "Brisket"),
            other => panic!("expected parsed label, got {other:?}"),
        }

        // Valid Homie RGB color.
        let color = prop_ref(&device_id, "probe-1", BBQ_PROBE_NODE_COLOR_PROP_ID);
        match publisher.parse_set(&color, &desc, "rgb,255,128,0") {
            ParseOutcome::Parsed(BbqProbeNodeSetEvents::Color(HomieColorValue::RGB(r, g, b))) => {
                assert_eq!((r, g, b), (255, 128, 0));
            }
            other => panic!("expected parsed color, got {other:?}"),
        }
        assert!(matches!(
            publisher.parse_set(&color, &desc, "#ff8000"),
            ParseOutcome::Invalid(_)
        ));

        // Unrelated property on the same node.
        let unrelated = prop_ref(&device_id, "probe-1", HomieID::new_const("unrelated"));
        assert!(matches!(
            publisher.parse_set(&unrelated, &desc, "1"),
            ParseOutcome::NoMatch
        ));

        // Same property on a different node must not match.
        let other_node = prop_ref(&device_id, "probe-2", BBQ_PROBE_NODE_LOW_THRESHOLD_PROP_ID);
        assert!(matches!(
            publisher.parse_set(&other_node, &desc, "55.5"),
            ParseOutcome::NoMatch
        ));
    }

    #[test]
    fn non_settable_properties_do_not_match_set_commands() {
        let device_id: HomieID = "bbq-test-device".try_into().unwrap();
        let (client, _) = Homie5DeviceProtocol::new(device_id.clone(), HomieDomain::Default);
        let node_id: HomieID = "probe-1".try_into().unwrap();

        let mut config = full_config();
        config.sensor_type_settable = false;
        let (node_desc, publisher) =
            BbqProbeNodeBuilder::new(&config).build_with_publisher(node_id.clone(), &client);
        let desc = DeviceDescriptionBuilder::new()
            .name("bbq test")
            .add_node(node_id, node_desc)
            .build();

        let sensor = prop_ref(&device_id, "probe-1", BBQ_PROBE_NODE_SENSOR_TYPE_PROP_ID);
        assert!(matches!(
            publisher.parse_set(&sensor, &desc, "iGrill2"),
            ParseOutcome::NoMatch
        ));
    }

    #[test]
    fn optional_publisher_methods_return_none_when_not_configured() {
        let device_id: HomieID = "bbq-test-device".try_into().unwrap();
        let (client, _) = Homie5DeviceProtocol::new(device_id, HomieDomain::Default);
        let node_id: HomieID = "probe-1".try_into().unwrap();
        let (_, publisher) = BbqProbeNodeBuilder::new(&base_disabled_config())
            .build_with_publisher(node_id, &client);

        assert!(publisher.low_threshold(50.0).is_none());
        assert!(publisher.low_threshold_target(50.0).is_none());
        assert!(publisher.high_threshold(120.0).is_none());
        assert!(publisher.high_threshold_target(120.0).is_none());
        assert!(
            publisher
                .threshold_state(BbqProbeThresholdState::Normal)
                .is_none()
        );
        assert!(publisher.role("pit").is_none());
        assert!(publisher.role_target("pit").is_none());
        assert!(publisher.label("Brisket").is_none());
        assert!(publisher.label_target("Brisket").is_none());
        assert!(publisher.alarm_mode("off").is_none());
        assert!(publisher.alarm_mode_target("off").is_none());
        assert!(publisher.color(HomieColorValue::RGB(1, 2, 3)).is_none());
        assert!(
            publisher
                .color_target(HomieColorValue::RGB(1, 2, 3))
                .is_none()
        );
        assert!(publisher.sensor_type("iGrill2").is_none());
        assert!(publisher.sensor_type_target("iGrill2").is_none());
    }

    #[test]
    fn configured_publisher_methods_produce_publishes() {
        let device_id: HomieID = "bbq-test-device".try_into().unwrap();
        let (client, _) = Homie5DeviceProtocol::new(device_id, HomieDomain::Default);
        let node_id: HomieID = "probe-1".try_into().unwrap();
        let (_, publisher) =
            BbqProbeNodeBuilder::new(&full_config()).build_with_publisher(node_id, &client);

        let temp = publisher.temperature(98.6);
        assert!(temp.topic.ends_with("probe-1/temperature"));
        assert!(temp.retain);
        assert_eq!(temp.payload, b"98.6");

        let connected = publisher.connected(true);
        assert!(connected.topic.ends_with("probe-1/connected"));
        assert_eq!(connected.payload, b"true");

        let color = publisher
            .color(HomieColorValue::RGB(255, 128, 0))
            .expect("color is configured");
        assert_eq!(color.payload, b"rgb,255,128,0");

        let state = publisher
            .threshold_state(BbqProbeThresholdState::High)
            .expect("threshold-state is configured");
        assert_eq!(state.payload, b"high");

        let target = publisher
            .low_threshold_target(55.0)
            .expect("thresholds are configured");
        assert!(target.topic.ends_with("probe-1/low-threshold/$target"));
    }

    #[test]
    fn empty_config_deserializes_to_default() {
        let config: BbqProbeNodeConfig =
            serde_json::from_str("{}").expect("bbq-probe config must deserialize");
        assert_eq!(config, BbqProbeNodeConfig::default());
    }
}

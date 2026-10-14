use std::fmt;

use homie5::{
    Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, IntegerRange, NodeDescriptionBuilder, PropertyDescriptionBuilder,
        PropertyDescriptionValidationError,
    },
};
use serde::{Deserialize, Serialize};

use crate::{PropSpec, SMARTHOME_CAP_SELECTOR};

pub const SELECTOR_NODE_DEFAULT_ID: HomieID = HomieID::new_const("selector");
pub const SELECTOR_NODE_DEFAULT_NAME: &str = "Selector";

pub const SELECTOR_NODE_POSITION_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("position"),
    retained: true,
    settable: false,
};
pub const SELECTOR_NODE_LABEL_PROP: PropSpec = PropSpec {
    id: HomieID::new_const("label"),
    retained: true,
    settable: false,
};

// ── Node (state) ────────────────────────────────────────────────────────────

/// A discrete position chosen by the user: a multi-position selector switch, a mode
/// knob with detents, or the face of a cube that points up.
#[derive(Debug)]
pub struct SelectorNode {
    pub publisher: SelectorNodePublisher,
    pub position: Option<i64>,
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SelectorNodeConfig {
    /// Lowest position, inclusive.
    pub min: i64,
    /// Highest position, inclusive.
    pub max: i64,
    /// Optional descriptive names, one per position from `min` to `max`, in order.
    /// When present, the node exposes the `label` property.
    pub labels: Vec<String>,
}

impl Default for SelectorNodeConfig {
    fn default() -> Self {
        Self {
            min: 1,
            max: 2,
            labels: Vec::new(),
        }
    }
}

impl SelectorNodeConfig {
    /// Number of positions between `min` and `max`, inclusive.
    pub fn position_count(&self) -> Option<u64> {
        let count = i128::from(self.max) - i128::from(self.min) + 1;
        u64::try_from(count).ok().filter(|count| *count > 0)
    }

    /// The label configured for `position`, if labels are configured and the
    /// position is within range.
    pub fn label_for(&self, position: i64) -> Option<&str> {
        if !(self.min..=self.max).contains(&position) {
            return None;
        }
        let index = usize::try_from(i128::from(position) - i128::from(self.min)).ok()?;
        self.labels.get(index).map(String::as_str)
    }

    pub fn validate(&self) -> Result<(), SelectorConfigError> {
        let Some(positions) = self.position_count() else {
            return Err(SelectorConfigError::InvalidRange {
                min: self.min,
                max: self.max,
            });
        };
        if self.labels.is_empty() {
            return Ok(());
        }
        if u64::try_from(self.labels.len()).ok() != Some(positions) {
            return Err(SelectorConfigError::LabelCountMismatch {
                positions,
                labels: self.labels.len(),
            });
        }
        PropertyDescriptionBuilder::enumeration(self.labels.iter().map(String::as_str))
            .map(|_| ())
            .map_err(SelectorConfigError::InvalidLabels)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorConfigError {
    /// `min` is greater than `max`.
    InvalidRange { min: i64, max: i64 },
    /// `labels` is not empty and its length differs from the number of positions.
    LabelCountMismatch { positions: u64, labels: usize },
    /// The labels are not a valid Homie enum format (empty or duplicate values).
    InvalidLabels(PropertyDescriptionValidationError),
}

impl fmt::Display for SelectorConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectorConfigError::InvalidRange { min, max } => {
                write!(
                    f,
                    "selector range is empty: min {min} is greater than max {max}"
                )
            }
            SelectorConfigError::LabelCountMismatch { positions, labels } => write!(
                f,
                "selector has {positions} positions but {labels} labels; labels must cover every position"
            ),
            SelectorConfigError::InvalidLabels(err) => write!(f, "invalid selector labels: {err}"),
        }
    }
}

impl std::error::Error for SelectorConfigError {}

// ── Builder ─────────────────────────────────────────────────────────────────

pub struct SelectorNodeBuilder {
    config: SelectorNodeConfig,
    node_builder: NodeDescriptionBuilder,
}

impl SelectorNodeBuilder {
    /// Creates the builder; rejects configs whose labels don't match the position range.
    pub fn new(config: &SelectorNodeConfig) -> Result<Self, SelectorConfigError> {
        config.validate()?;
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(SELECTOR_NODE_DEFAULT_NAME),
            config,
        )
        .r#type(SMARTHOME_CAP_SELECTOR);

        Ok(Self {
            node_builder: db,
            config: config.clone(),
        })
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &SelectorNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property(
            SELECTOR_NODE_POSITION_PROP.id,
            PropertyDescriptionBuilder::integer()
                .name("Position")
                .integer_range(IntegerRange {
                    min: Some(config.min),
                    max: Some(config.max),
                    step: None,
                })
                .settable(SELECTOR_NODE_POSITION_PROP.settable)
                .retained(SELECTOR_NODE_POSITION_PROP.retained)
                .build(),
        )
        .add_property_cond(
            SELECTOR_NODE_LABEL_PROP.id,
            !config.labels.is_empty(),
            || {
                PropertyDescriptionBuilder::enumeration(config.labels.iter().map(String::as_str))
                    .unwrap()
                    .name("Position label")
                    .settable(SELECTOR_NODE_LABEL_PROP.settable)
                    .retained(SELECTOR_NODE_LABEL_PROP.retained)
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
    ) -> (HomieNodeDescription, SelectorNodePublisher) {
        (
            self.node_builder.build(),
            SelectorNodePublisher::new(
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
pub struct SelectorNodePublisher {
    client: Homie5DeviceProtocol,
    config: SelectorNodeConfig,
    node: NodeRef,
}

impl SelectorNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol, config: SelectorNodeConfig) -> Self {
        Self {
            node,
            client,
            config,
        }
    }

    /// All messages for a position change: `position`, followed by `label` when
    /// labels are configured. `None` if `position` is outside the configured range.
    pub fn position(&self, position: i64) -> Option<Vec<homie5::client::Publish>> {
        if !(self.config.min..=self.config.max).contains(&position) {
            return None;
        }
        let mut publishes = vec![self.client.publish_value(
            self.node.node_id(),
            &SELECTOR_NODE_POSITION_PROP.id,
            position.to_string(),
            SELECTOR_NODE_POSITION_PROP.retained,
        )];
        if let Some(label) = self.config.label_for(position) {
            publishes.push(self.client.publish_value(
                self.node.node_id(),
                &SELECTOR_NODE_LABEL_PROP.id,
                label,
                SELECTOR_NODE_LABEL_PROP.retained,
            ));
        }
        Some(publishes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use homie5::{HomieDomain, client::QoS, device_description::HomiePropertyFormat};

    fn knob_config() -> SelectorNodeConfig {
        SelectorNodeConfig {
            min: 0,
            max: 3,
            labels: vec!["off".into(), "low".into(), "medium".into(), "high".into()],
        }
    }

    #[test]
    fn position_range_comes_from_config_and_label_is_optional() {
        let cube = SelectorNodeConfig {
            min: 1,
            max: 6,
            labels: Vec::new(),
        };
        let node = SelectorNodeBuilder::new(&cube).unwrap().build();
        let position = node
            .properties
            .get(&SELECTOR_NODE_POSITION_PROP.id)
            .unwrap();

        assert_eq!(node.r#type.as_deref(), Some(SMARTHOME_CAP_SELECTOR));
        assert_eq!(
            position.format,
            HomiePropertyFormat::IntegerRange(IntegerRange {
                min: Some(1),
                max: Some(6),
                step: None,
            })
        );
        assert!(position.retained);
        assert!(!position.settable);
        assert!(!node.properties.contains_key(&SELECTOR_NODE_LABEL_PROP.id));
    }

    #[test]
    fn labels_become_a_retained_enum_in_position_order() {
        let node = SelectorNodeBuilder::new(&knob_config()).unwrap().build();
        let label = node.properties.get(&SELECTOR_NODE_LABEL_PROP.id).unwrap();

        assert!(label.retained);
        assert!(!label.settable);
        assert_eq!(
            label.format,
            HomiePropertyFormat::Enum(vec![
                "off".to_string(),
                "low".to_string(),
                "medium".to_string(),
                "high".to_string(),
            ])
        );
    }

    #[test]
    fn builder_rejects_label_count_mismatch_and_invalid_ranges() {
        let mut config = knob_config();
        config.labels.pop();
        assert_eq!(
            SelectorNodeBuilder::new(&config).err(),
            Some(SelectorConfigError::LabelCountMismatch {
                positions: 4,
                labels: 3,
            })
        );

        let inverted = SelectorNodeConfig {
            min: 5,
            max: 1,
            labels: Vec::new(),
        };
        assert_eq!(
            SelectorNodeBuilder::new(&inverted).err(),
            Some(SelectorConfigError::InvalidRange { min: 5, max: 1 })
        );

        let duplicates = SelectorNodeConfig {
            min: 1,
            max: 2,
            labels: vec!["same".into(), "same".into()],
        };
        assert_eq!(
            SelectorNodeBuilder::new(&duplicates).err(),
            Some(SelectorConfigError::InvalidLabels(
                PropertyDescriptionValidationError::DuplicateEnumValues
            ))
        );
    }

    #[test]
    fn position_publishes_position_and_label_retained() {
        let (client, _) =
            Homie5DeviceProtocol::new(HomieID::new_const("dev"), HomieDomain::Default);
        let (_, publisher) = SelectorNodeBuilder::new(&knob_config())
            .unwrap()
            .build_with_publisher(SELECTOR_NODE_DEFAULT_ID, &client);

        let publishes = publisher.position(2).unwrap();
        assert_eq!(publishes.len(), 2);
        assert!(publishes[0].topic.ends_with("/selector/position"));
        assert_eq!(publishes[0].payload, b"2");
        assert!(publishes[1].topic.ends_with("/selector/label"));
        assert_eq!(publishes[1].payload, b"medium");
        for publish in &publishes {
            assert!(publish.retain);
            assert_eq!(publish.qos, QoS::ExactlyOnce);
        }

        assert!(publisher.position(4).is_none());
        assert!(publisher.position(-1).is_none());
    }

    #[test]
    fn without_labels_only_position_is_published() {
        let (client, _) =
            Homie5DeviceProtocol::new(HomieID::new_const("dev"), HomieDomain::Default);
        let (_, publisher) = SelectorNodeBuilder::new(&SelectorNodeConfig::default())
            .unwrap()
            .build_with_publisher(SELECTOR_NODE_DEFAULT_ID, &client);

        let publishes = publisher.position(1).unwrap();
        assert_eq!(publishes.len(), 1);
        assert_eq!(publishes[0].payload, b"1");
    }
}

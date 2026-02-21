use chrono::prelude::*;

use homie5::{
    Homie5DeviceProtocol, HomieID, NodeRef,
    device_description::{
        HomieNodeDescription, NodeDescriptionBuilder, PropertyDescriptionBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::SMARTHOME_TYPE_MAINTENANCE;

pub const MAINTENANCE_NODE_DEFAULT_ID: &str = "maintenance";
pub const MAINTENANCE_NODE_DEFAULT_NAME: &str = "Maintenance information";
pub const MAINTENANCE_NODE_LOW_BATTERY_PROP_ID: &str = "low-battery";
pub const MAINTENANCE_NODE_BATTERY_LEVEL_PROP_ID: &str = "battery-level";
pub const MAINTENANCE_NODE_LAST_UPDATE_PROP_ID: &str = "last-update";
pub const MAINTENANCE_NODE_REACHABLE_PROP_ID: &str = "reachable";

#[derive(Debug)]
pub struct MaintenanceNode {
    pub publisher: MaintenanceNodePublisher,
    pub battery_level: Option<i64>,
    pub low_battery: Option<bool>,
    pub last_update: Option<DateTime<Utc>>,
    pub reachable: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MaintenanceNodeConfig {
    pub low_battery: bool,
    pub battery_level: bool,
    pub reachable: bool,
    pub last_update: bool,
}

impl Default for MaintenanceNodeConfig {
    fn default() -> Self {
        Self {
            battery_level: false,
            low_battery: true,
            reachable: true,
            last_update: true,
        }
    }
}

pub struct MaintenanceNodeBuilder {
    config: MaintenanceNodeConfig,
    node_builder: NodeDescriptionBuilder,
}

impl MaintenanceNodeBuilder {
    pub fn new(config: MaintenanceNodeConfig) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(MAINTENANCE_NODE_DEFAULT_NAME),
            &config,
        )
        .r#type(SMARTHOME_TYPE_MAINTENANCE);

        Self {
            node_builder: db,
            config,
        }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        config: &MaintenanceNodeConfig,
    ) -> NodeDescriptionBuilder {
        db.add_property_cond(
            MAINTENANCE_NODE_LOW_BATTERY_PROP_ID.try_into().unwrap(),
            config.low_battery,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                    .name("Low battery indicator")
                    .settable(false)
                    .retained(true)
                    .build()
            },
        )
        .add_property_cond(
            MAINTENANCE_NODE_BATTERY_LEVEL_PROP_ID.try_into().unwrap(),
            config.battery_level,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
                    .name("Battery level")
                    .settable(false)
                    .retained(true)
                    .build()
            },
        )
        .add_property_cond(
            MAINTENANCE_NODE_LAST_UPDATE_PROP_ID.try_into().unwrap(),
            config.last_update,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Datetime)
                    .name("Last update")
                    .settable(false)
                    .retained(true)
                    .build()
            },
        )
        .add_property_cond(
            MAINTENANCE_NODE_REACHABLE_PROP_ID.try_into().unwrap(),
            config.reachable,
            || {
                PropertyDescriptionBuilder::new(homie5::HomieDataType::Boolean)
                    .name("Reachable")
                    .settable(false)
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
    ) -> (HomieNodeDescription, MaintenanceNodePublisher) {
        (
            self.node_builder.build(),
            MaintenanceNodePublisher::new(
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

#[derive(Debug)]
pub struct MaintenanceNodePublisher {
    client: Homie5DeviceProtocol,
    config: MaintenanceNodeConfig,
    node: NodeRef,
    low_battery_prop: HomieID,
    battery_level_prop: HomieID,
    last_update_prop: HomieID,
    reachable_prop: HomieID,
}

impl MaintenanceNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol, config: MaintenanceNodeConfig) -> Self {
        Self {
            node,
            client,
            config,
            low_battery_prop: MAINTENANCE_NODE_LOW_BATTERY_PROP_ID.try_into().unwrap(),
            battery_level_prop: MAINTENANCE_NODE_BATTERY_LEVEL_PROP_ID.try_into().unwrap(),
            last_update_prop: MAINTENANCE_NODE_LAST_UPDATE_PROP_ID.try_into().unwrap(),
            reachable_prop: MAINTENANCE_NODE_REACHABLE_PROP_ID.try_into().unwrap(),
        }
    }

    pub fn low_battery(&self, value: bool) -> Option<homie5::client::Publish> {
        if !self.config.low_battery {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &self.low_battery_prop,
            value.to_string(),
            true,
        ))
    }

    pub fn battery_level(&self, value: i32) -> Option<homie5::client::Publish> {
        if !self.config.battery_level {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &self.battery_level_prop,
            value.to_string(),
            true,
        ))
    }
    pub fn last_update(&self, value: DateTime<Utc>) -> Option<homie5::client::Publish> {
        if !self.config.last_update {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &self.last_update_prop,
            value.to_rfc3339_opts(SecondsFormat::Millis, true),
            true,
        ))
    }
    pub fn reachable(&self, value: bool) -> Option<homie5::client::Publish> {
        if !self.config.reachable {
            return None;
        }
        Some(self.client.publish_value(
            self.node.node_id(),
            &self.reachable_prop,
            value.to_string(),
            true,
        ))
    }
}

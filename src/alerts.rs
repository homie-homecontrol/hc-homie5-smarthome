//! Well-known alert IDs for the homie-homecontrol smarthome specification.
//!
//! Alert IDs follow Homie topic ID rules (lowercase `a-z`, `0-9`, `-` only)
//! and use the `hc-` prefix to namespace them within the homecontrol ecosystem.
//!
//! Devices may publish custom alert IDs without the `hc-` prefix.
//! Controllers should render recognised IDs with specialised icons/labels
//! and fall back to a generic warning presentation for unknown IDs.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

// ── Alert ID constants ──────────────────────────────────────────────────────

/// Battery level is low.
pub const HC_ALERT_BATTERY_LOW: &str = "hc-battery-low";

/// Battery level is critically low.
pub const HC_ALERT_BATTERY_CRITICAL: &str = "hc-battery-critical";

/// Device is unreachable on the underlying network/protocol.
pub const HC_ALERT_UNREACHABLE: &str = "hc-unreachable";

/// No update has been received from the device for an extended period.
pub const HC_ALERT_UPDATE_OVERDUE: &str = "hc-update-overdue";

/// The device has a configuration error.
pub const HC_ALERT_CONFIG_ERROR: &str = "hc-config-error";

/// A sensor on the device is reporting faulty or out-of-range readings.
pub const HC_ALERT_SENSOR_FAULT: &str = "hc-sensor-fault";

/// Physical tamper detected on the device.
pub const HC_ALERT_TAMPER: &str = "hc-tamper";

/// Communication error with the underlying protocol (e.g. ZWave, Zigbee).
pub const HC_ALERT_COMM_ERROR: &str = "hc-comm-error";

// ── Enum ────────────────────────────────────────────────────────────────────

/// Typed representation of the well-known homecontrol alert IDs.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SmarthomeAlert {
    BatteryLow,
    BatteryCritical,
    Unreachable,
    UpdateOverdue,
    ConfigError,
    SensorFault,
    Tamper,
    CommError,
}

impl SmarthomeAlert {
    /// Return the Homie topic-ID string for this alert.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::BatteryLow => HC_ALERT_BATTERY_LOW,
            Self::BatteryCritical => HC_ALERT_BATTERY_CRITICAL,
            Self::Unreachable => HC_ALERT_UNREACHABLE,
            Self::UpdateOverdue => HC_ALERT_UPDATE_OVERDUE,
            Self::ConfigError => HC_ALERT_CONFIG_ERROR,
            Self::SensorFault => HC_ALERT_SENSOR_FAULT,
            Self::Tamper => HC_ALERT_TAMPER,
            Self::CommError => HC_ALERT_COMM_ERROR,
        }
    }

    /// Try to parse an alert ID string into a well-known variant.
    /// Returns `None` for unrecognised (custom) alert IDs.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            HC_ALERT_BATTERY_LOW => Some(Self::BatteryLow),
            HC_ALERT_BATTERY_CRITICAL => Some(Self::BatteryCritical),
            HC_ALERT_UNREACHABLE => Some(Self::Unreachable),
            HC_ALERT_UPDATE_OVERDUE => Some(Self::UpdateOverdue),
            HC_ALERT_CONFIG_ERROR => Some(Self::ConfigError),
            HC_ALERT_SENSOR_FAULT => Some(Self::SensorFault),
            HC_ALERT_TAMPER => Some(Self::Tamper),
            HC_ALERT_COMM_ERROR => Some(Self::CommError),
            _ => None,
        }
    }
}

impl FromStr for SmarthomeAlert {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_id(s).ok_or(())
    }
}

impl fmt::Display for SmarthomeAlert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        for alert in [
            SmarthomeAlert::BatteryLow,
            SmarthomeAlert::BatteryCritical,
            SmarthomeAlert::Unreachable,
            SmarthomeAlert::UpdateOverdue,
            SmarthomeAlert::ConfigError,
            SmarthomeAlert::SensorFault,
            SmarthomeAlert::Tamper,
            SmarthomeAlert::CommError,
        ] {
            let s = alert.as_str();
            let parsed = SmarthomeAlert::from_id(s).expect("should parse");
            assert_eq!(alert, parsed);
        }
    }

    #[test]
    fn test_unknown_returns_none() {
        assert_eq!(SmarthomeAlert::from_id("custom-alert"), None);
        assert_eq!(SmarthomeAlert::from_id("battery-low"), None); // missing hc- prefix
    }
}

pub mod button_node;
pub mod colorlight_node;
pub mod contact_node;
pub mod dimmer_node;
pub mod light_scene_node;
pub mod maintenance_node;
pub mod motion_node;
pub mod numeric_sensor_node;
pub mod orientation_node;
pub mod shutter_node;
pub mod switch_node;
pub mod thermostat_node;
pub mod tilt_node;
pub mod vibration_node;
pub mod water_sensor_node;
pub mod weather_node;

use std::{fmt, str::FromStr};

use button_node::ButtonNodeConfig;
use colorlight_node::{ColorlightNode, ColorlightNodeConfig};
use contact_node::ContactNode;
use dimmer_node::{DimmerNode, DimmerNodeConfig};
use light_scene_node::LightSceneNodeConfig;
use maintenance_node::{MaintenanceNode, MaintenanceNodeConfig};
use motion_node::{MotionNode, MotionNodeConfig};
use numeric_sensor_node::NumericSensorNode;
use serde::{Deserialize, Serialize};
use shutter_node::{ShutterNode, ShutterNodeConfig};
use switch_node::{SwitchNode, SwitchNodeConfig};
use thermostat_node::ThermostatNodeConfig;
use tilt_node::TiltNode;
use vibration_node::VibrationNodeConfig;
use water_sensor_node::WaterSensorNode;
use weather_node::{WeatherNode, WeatherNodeConfig};

/// Helper macro to generate static smarthome type strings
macro_rules! create_smarthome_type {
    ($type:expr) => {
        concat!("homie-homecontrol/v1/type=", $type)
    };
}

/// Helper macro to generate static smarthome type strings for extensions
#[macro_export]
macro_rules! create_smarthome_type_extension {
    ($type:expr) => {
        concat!("homie-homecontrol/v1/extension/type=", $type)
    };
}

pub const SMARTHOME_NS_V1: &str = "homie-homecontrol/v1";

pub const SMARTHOME_TYPE_MAINTENANCE: &str = create_smarthome_type!("maintenance");
pub const SMARTHOME_TYPE_SWITCH: &str = create_smarthome_type!("switch");
pub const SMARTHOME_TYPE_DIMMER: &str = create_smarthome_type!("dimmer");
pub const SMARTHOME_TYPE_CONTACT: &str = create_smarthome_type!("contact");
pub const SMARTHOME_TYPE_WEATHER: &str = create_smarthome_type!("weather");
pub const SMARTHOME_TYPE_MOTION: &str = create_smarthome_type!("motion");
pub const SMARTHOME_TYPE_BUTTON: &str = create_smarthome_type!("button");
pub const SMARTHOME_TYPE_COLORLIGHT: &str = create_smarthome_type!("colorlight");
pub const SMARTHOME_TYPE_LIGHTSCENE: &str = create_smarthome_type!("lightscene");
pub const SMARTHOME_TYPE_NUMERIC: &str = create_smarthome_type!("numeric");
pub const SMARTHOME_TYPE_VIBRATION: &str = create_smarthome_type!("vibration");
pub const SMARTHOME_TYPE_ORIENTATION: &str = create_smarthome_type!("orientation");
pub const SMARTHOME_TYPE_WATER_SENSOR: &str = create_smarthome_type!("water");
pub const SMARTHOME_TYPE_SHUTTER: &str = create_smarthome_type!("shutter");
pub const SMARTHOME_TYPE_TILT: &str = create_smarthome_type!("tilt");
pub const SMARTHOME_TYPE_THERMOSTAT: &str = create_smarthome_type!("thermostat");

/// SmarthomeType enum representing various smart home device types.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] // Ensures consistent lowercase naming in serialization/deserialization
pub enum SmarthomeType {
    Switch,
    Dimmer,
    Maintenance,
    Contact,
    Weather,
    Motion,
    Button,
    ColorLight,
    LightScene,
    Numeric,
    Vibration,
    Orientation,
    WaterSensor,
    Shutter,
    Tilt,
    Thermostat,
}

impl SmarthomeType {
    /// Convert the enum variant into its corresponding string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            SmarthomeType::Switch => SMARTHOME_TYPE_SWITCH,
            SmarthomeType::Dimmer => SMARTHOME_TYPE_DIMMER,
            SmarthomeType::Maintenance => SMARTHOME_TYPE_MAINTENANCE,
            SmarthomeType::Contact => SMARTHOME_TYPE_CONTACT,
            SmarthomeType::Weather => SMARTHOME_TYPE_WEATHER,
            SmarthomeType::Motion => SMARTHOME_TYPE_MOTION,
            SmarthomeType::Button => SMARTHOME_TYPE_BUTTON,
            SmarthomeType::ColorLight => SMARTHOME_TYPE_COLORLIGHT,
            SmarthomeType::LightScene => SMARTHOME_TYPE_LIGHTSCENE,
            SmarthomeType::Numeric => SMARTHOME_TYPE_NUMERIC,
            SmarthomeType::Vibration => SMARTHOME_TYPE_VIBRATION,
            SmarthomeType::Orientation => SMARTHOME_TYPE_ORIENTATION,
            SmarthomeType::WaterSensor => SMARTHOME_TYPE_WATER_SENSOR,
            SmarthomeType::Shutter => SMARTHOME_TYPE_SHUTTER,
            SmarthomeType::Tilt => SMARTHOME_TYPE_TILT,
            SmarthomeType::Thermostat => SMARTHOME_TYPE_THERMOSTAT,
        }
    }

    /// Create a SmarthomeType from a string containing a constant value.
    pub fn from_constant(value: &str) -> Option<Self> {
        match value {
            SMARTHOME_TYPE_SWITCH => Some(SmarthomeType::Switch),
            SMARTHOME_TYPE_DIMMER => Some(SmarthomeType::Dimmer),
            SMARTHOME_TYPE_MAINTENANCE => Some(SmarthomeType::Maintenance),
            SMARTHOME_TYPE_CONTACT => Some(SmarthomeType::Contact),
            SMARTHOME_TYPE_WEATHER => Some(SmarthomeType::Weather),
            SMARTHOME_TYPE_MOTION => Some(SmarthomeType::Motion),
            SMARTHOME_TYPE_BUTTON => Some(SmarthomeType::Button),
            SMARTHOME_TYPE_COLORLIGHT => Some(SmarthomeType::ColorLight),
            SMARTHOME_TYPE_LIGHTSCENE => Some(SmarthomeType::LightScene),
            SMARTHOME_TYPE_NUMERIC => Some(SmarthomeType::Numeric),
            SMARTHOME_TYPE_VIBRATION => Some(SmarthomeType::Vibration),
            SMARTHOME_TYPE_ORIENTATION => Some(SmarthomeType::Orientation),
            SMARTHOME_TYPE_WATER_SENSOR => Some(SmarthomeType::WaterSensor),
            SMARTHOME_TYPE_SHUTTER => Some(SmarthomeType::Shutter),
            SMARTHOME_TYPE_TILT => Some(SmarthomeType::Tilt),
            SMARTHOME_TYPE_THERMOSTAT => Some(SmarthomeType::Thermostat),
            _ => None,
        }
    }
}

/// Implement `FromStr` to parse a SmarthomeType from a string.
impl FromStr for SmarthomeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SmarthomeType::from_constant(s).ok_or(())
    }
}

/// Implement `Display` for better formatting and output.
impl fmt::Display for SmarthomeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SmarthomeProperyConfig {
    Button(ButtonNodeConfig),
    ColorLight(ColorlightNodeConfig),
    Dimmer(DimmerNodeConfig),
    LightScene(LightSceneNodeConfig),
    Maintenance(MaintenanceNodeConfig),
    Motion(MotionNodeConfig),
    Shutter(ShutterNodeConfig),
    Switch(SwitchNodeConfig),
    Thermostat(ThermostatNodeConfig),
    Vibration(VibrationNodeConfig),
    Weather(WeatherNodeConfig),
}

#[derive(Debug)]
pub enum SmarthomeNode {
    MaintenanceNode(MaintenanceNode),
    SwitchNode(SwitchNode),
    DimmerNode(DimmerNode),
    WeatherNode(WeatherNode),
    ContactNode(ContactNode),
    MotionNode(MotionNode),
    ColorlightNode(ColorlightNode),
    NumericSensorNode(NumericSensorNode),
    WaterSensor(WaterSensorNode),
    ShutterNode(ShutterNode),
    TiltNode(TiltNode),
}

#[cfg(test)]
mod tests {
    use rumqttc::{AsyncClient, ClientError};
    use std::{env, time::Duration};
    use tokio::sync::mpsc::channel;

    use homie5::{
        client::{Publish, Subscription},
        device_description::DeviceDescriptionBuilder,
        parse_mqtt_message, Homie5DeviceProtocol, Homie5Message, HomieDeviceStatus, HomieDomain,
        HomieID,
    };

    use crate::{
        dimmer_node::{DimmerNodeBuilder, DIMMER_NODE_DEFAULT_ID},
        maintenance_node::{MaintenanceNodeBuilder, MAINTENANCE_NODE_DEFAULT_ID},
        switch_node::{
            SwitchNodeActions, SwitchNodeBuilder, SwitchNodeSetEvents, SWITCH_NODE_DEFAULT_ID,
        },
        weather_node::{WeatherNodeBuilder, WEATHER_NODE_DEFAULT_ID},
    };
    #[allow(clippy::large_enum_variant)]
    #[derive(Debug)]
    pub enum ClientEvent {
        Homie(Homie5Message),
        Mqtt(rumqttc::Event),
    }

    struct Settings {
        hostname: String,
        port: u16,
        username: String,
        password: String,
        client_id: String,
        homie_domain: HomieDomain,
    }

    fn get_settings() -> Settings {
        let hostname = env::var("HOMIE_MQTT_HOST").unwrap_or_default();

        let port = if let Ok(port) = env::var("HOMIE_MQTT_PORT") {
            port.parse::<u16>().expect("Not a valid number for port!")
        } else {
            1883
        };

        let username = env::var("HOMIE_MQTT_USERNAME").unwrap_or_default();

        let password = env::var("HOMIE_MQTT_PASSWORD").unwrap_or_default();

        let client_id = if let Ok(client_id) = env::var("HOMIE_MQTT_CLIENT_ID") {
            client_id
        } else {
            String::from("aslkdnlauidhwwkednwek")
        };
        let homie_domain = if let Ok(homie_domain) = env::var("HOMIE_MQTT_HOMIE_DOMAIN") {
            homie_domain.try_into().unwrap_or_default()
        } else {
            HomieDomain::Default
        };

        Settings {
            hostname,
            port,
            username,
            password,
            client_id,
            homie_domain,
        }
    }

    fn qos_to_rumqttc(value: homie5::client::QoS) -> rumqttc::QoS {
        match value {
            homie5::client::QoS::AtLeastOnce => rumqttc::QoS::AtLeastOnce,
            homie5::client::QoS::AtMostOnce => rumqttc::QoS::AtMostOnce,
            homie5::client::QoS::ExactlyOnce => rumqttc::QoS::ExactlyOnce,
        }
    }
    fn lw_to_rumqttc(value: homie5::client::LastWill) -> rumqttc::LastWill {
        rumqttc::LastWill {
            topic: value.topic,
            message: value.message.into(),
            qos: qos_to_rumqttc(value.qos),
            retain: value.retain,
        }
    }

    async fn publish(client: &AsyncClient, p: Publish) -> Result<(), ClientError> {
        client
            .publish(p.topic, qos_to_rumqttc(p.qos), p.retain, p.payload)
            .await
    }

    async fn subscribe(
        client: &AsyncClient,
        subs: impl Iterator<Item = Subscription>,
    ) -> Result<(), ClientError> {
        for sub in subs {
            client.subscribe(sub.topic, qos_to_rumqttc(sub.qos)).await?;
        }
        Ok(())
    }
    #[tokio::test]
    #[ignore]
    async fn test_device() {
        let _settings = get_settings();
        let mut mqttoptions = rumqttc::MqttOptions::new(
            _settings.client_id + "_dev",
            _settings.hostname,
            _settings.port,
        );
        mqttoptions.set_credentials(_settings.username, _settings.password);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        mqttoptions.set_clean_session(true);

        let id: HomieID = "test-hc-smarthome-1".try_into().unwrap();
        let mut switch_state = false;
        let mut switch_state2 = false;
        let mut dimmer_state: i64 = 0;

        let (client, last_will) = Homie5DeviceProtocol::new(id.clone(), _settings.homie_domain);
        mqttoptions.set_last_will(lw_to_rumqttc(last_will));

        let (mqtt_client, mut eventloop) = rumqttc::AsyncClient::new(mqttoptions, 65535);

        let (channel_tx, mut channel_rx) = channel(65535);

        let _handle = tokio::task::spawn(async move {
            loop {
                let event = eventloop.poll().await;

                match event {
                    Ok(event) => {
                        let event = match &event {
                            rumqttc::Event::Incoming(rumqttc::Packet::Publish(p)) => {
                                if let Ok(event) = parse_mqtt_message(&p.topic, &p.payload) {
                                    ClientEvent::Homie(event)
                                } else {
                                    ClientEvent::Mqtt(event)
                                }
                            }
                            _ => ClientEvent::Mqtt(event),
                        };
                        let _ = channel_tx.send(event).await;
                    }

                    Err(err) => {
                        eprintln!("Error received from eventloop: {:#?}", err);
                    }
                }
            }
        });

        let (maintenance_node, maintenance_node_publisher) =
            MaintenanceNodeBuilder::new(Default::default())
                .build_with_publisher(MAINTENANCE_NODE_DEFAULT_ID.try_into().unwrap(), &client);

        let (switch_node, switch_node_publisher) = SwitchNodeBuilder::new(&Default::default())
            .build_with_publisher(SWITCH_NODE_DEFAULT_ID.try_into().unwrap(), &client);

        let (switch_node2, switch_node_publisher2) = SwitchNodeBuilder::new(&Default::default())
            .build_with_publisher("switch2".try_into().unwrap(), &client);

        let (dimmer_node, dimmer_node_publisher) = DimmerNodeBuilder::new(&Default::default())
            .build_with_publisher(DIMMER_NODE_DEFAULT_ID.try_into().unwrap(), &client);

        let (weather_node, weather_node_publisher) = WeatherNodeBuilder::new(&Default::default())
            .build_with_publisher(WEATHER_NODE_DEFAULT_ID.try_into().unwrap(), &client);

        let desc = DeviceDescriptionBuilder::new()
            .name("hc-smarthome-test")
            .add_node(
                MAINTENANCE_NODE_DEFAULT_ID.try_into().unwrap(),
                maintenance_node,
            )
            .add_node(SWITCH_NODE_DEFAULT_ID.try_into().unwrap(), switch_node)
            .add_node(DIMMER_NODE_DEFAULT_ID.try_into().unwrap(), dimmer_node)
            .add_node(WEATHER_NODE_DEFAULT_ID.try_into().unwrap(), weather_node)
            .add_node("switch2".try_into().unwrap(), switch_node2)
            .build();

        loop {
            let event_opt = channel_rx.recv().await;

            let event = match event_opt {
                Some(event) => event,
                None => {
                    continue;
                }
            };

            match &event {
                ClientEvent::Homie(event) => {
                    if let Homie5Message::PropertySet {
                        property: _,
                        set_value: _,
                    } = event
                    {
                        if let Some(switch_node_event) =
                            switch_node_publisher.match_parse_event(&desc, event)
                        {
                            println!("SwitchNode: {:#?}", switch_node_event);
                            match switch_node_event {
                                SwitchNodeSetEvents::State(swst) => {
                                    switch_state = swst;
                                    let _ = publish(
                                        &mqtt_client,
                                        switch_node_publisher.state_target(swst),
                                    )
                                    .await;

                                    // DO some actual change on a physical device here

                                    let _ =
                                        publish(&mqtt_client, switch_node_publisher.state(swst))
                                            .await;
                                }
                                SwitchNodeSetEvents::Action(action) => {
                                    match action {
                                        SwitchNodeActions::Toggle => {
                                            switch_state = !switch_state;
                                            let _ = publish(
                                                &mqtt_client,
                                                switch_node_publisher.state_target(switch_state),
                                            )
                                            .await;

                                            // DO some actual change on a physical device here

                                            let _ = publish(
                                                &mqtt_client,
                                                switch_node_publisher.state(switch_state),
                                            )
                                            .await;
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(switch_node_event) =
                            switch_node_publisher2.match_parse_event(&desc, event)
                        {
                            println!("SwitchNode2: {:#?}", switch_node_event);
                            match switch_node_event {
                                SwitchNodeSetEvents::State(swst) => {
                                    switch_state2 = swst;
                                    let _ = publish(
                                        &mqtt_client,
                                        switch_node_publisher2.state_target(switch_state2),
                                    )
                                    .await;

                                    // DO some actual change on a physical device here

                                    let _ = publish(
                                        &mqtt_client,
                                        switch_node_publisher2.state(switch_state2),
                                    )
                                    .await;
                                }
                                SwitchNodeSetEvents::Action(action) => {
                                    match action {
                                        SwitchNodeActions::Toggle => {
                                            switch_state2 = !switch_state2;
                                            let _ = publish(
                                                &mqtt_client,
                                                switch_node_publisher2.state_target(switch_state2),
                                            )
                                            .await;

                                            // DO some actual change on a physical device here

                                            let _ = publish(
                                                &mqtt_client,
                                                switch_node_publisher2.state(switch_state2),
                                            )
                                            .await;
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(dimmer_node_event) =
                            dimmer_node_publisher.match_parse_event(&desc, event)
                        {
                            println!("DimmerNode: {:#?}", dimmer_node_event);
                            match dimmer_node_event {
                                crate::dimmer_node::DimmerNodeSetEvents::Brightness(value) => {
                                    dimmer_state = value;

                                    let _ = publish(
                                        &mqtt_client,
                                        dimmer_node_publisher.brightness_target(dimmer_state),
                                    )
                                    .await;

                                    // DO some actual change on a physical device here

                                    let _ = publish(
                                        &mqtt_client,
                                        dimmer_node_publisher.brightness(dimmer_state),
                                    )
                                    .await;
                                }
                                crate::dimmer_node::DimmerNodeSetEvents::Action(action) => {
                                    match action {
                                        crate::dimmer_node::DimmerNodeActions::Brighter => {
                                            dimmer_state = std::cmp::min(dimmer_state + 10, 100);
                                        }
                                        crate::dimmer_node::DimmerNodeActions::Darker => {
                                            dimmer_state = std::cmp::max(dimmer_state - 10, 1);
                                        }
                                    }

                                    let _ = publish(
                                        &mqtt_client,
                                        dimmer_node_publisher.brightness_target(dimmer_state),
                                    )
                                    .await;

                                    // DO some actual change on a physical device here

                                    let _ = publish(
                                        &mqtt_client,
                                        dimmer_node_publisher.brightness(dimmer_state),
                                    )
                                    .await;
                                }
                            }
                        }
                        println!("Event: {:#?}", event);
                        println!("{}", chrono::Utc::now());
                    }
                }

                ClientEvent::Mqtt(event) => match &event {
                    rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(ca)) => {
                        println!("Connected! Publishing Device: {:#?}", ca);
                        let _ = publish(
                            &mqtt_client,
                            client.publish_state_for_id(&id, HomieDeviceStatus::Init),
                        )
                        .await;
                        let _ = publish(
                            &mqtt_client,
                            client.publish_description_for_id(&id, &desc).unwrap(),
                        )
                        .await;
                        let _ = subscribe(
                            &mqtt_client,
                            client.subscribe_props_for_id(&id, &desc).unwrap(),
                        )
                        .await;
                        let _ = publish(
                            &mqtt_client,
                            maintenance_node_publisher.low_battery(false).unwrap(),
                        )
                        .await;
                        let _ = publish(
                            &mqtt_client,
                            maintenance_node_publisher
                                .last_update(chrono::Utc::now())
                                .unwrap(),
                        )
                        .await;
                        let _ = publish(
                            &mqtt_client,
                            maintenance_node_publisher.reachable(true).unwrap(),
                        )
                        .await;
                        let _ = publish(
                            &mqtt_client,
                            switch_node_publisher.state_target(switch_state),
                        )
                        .await;
                        let _ =
                            publish(&mqtt_client, switch_node_publisher.state(switch_state)).await;
                        let _ = publish(
                            &mqtt_client,
                            switch_node_publisher2.state_target(switch_state2),
                        )
                        .await;
                        let _ = publish(&mqtt_client, switch_node_publisher2.state(switch_state2))
                            .await;
                        let _ =
                            publish(&mqtt_client, dimmer_node_publisher.brightness(dimmer_state))
                                .await;
                        let _ =
                            publish(&mqtt_client, weather_node_publisher.temperature(12.4)).await;
                        let _ = publish(&mqtt_client, weather_node_publisher.humidity(64)).await;

                        let _ = publish(
                            &mqtt_client,
                            client.publish_state_for_id(&id, HomieDeviceStatus::Ready),
                        )
                        .await;
                    }
                    rumqttc::Event::Incoming(rumqttc::Packet::Publish(p)) => {
                        println!("MQTT Publish: {:#?}", p);
                    }
                    _ => {}
                },
            }
        }
    }
}

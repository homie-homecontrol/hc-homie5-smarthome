use std::f64;

use homie5::{
    device_description::{
        FloatRange, HomieNodeDescription, HomiePropertyFormat, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
    Homie5DeviceProtocol, HomieID, NodeRef, HOMIE_UNIT_AMPERE, HOMIE_UNIT_DEGREE_CELSIUS,
    HOMIE_UNIT_HERTZ, HOMIE_UNIT_KILOPASCAL, HOMIE_UNIT_KILOWATTHOUR, HOMIE_UNIT_LITER,
    HOMIE_UNIT_LUX, HOMIE_UNIT_METER, HOMIE_UNIT_PERCENT, HOMIE_UNIT_VOLT, HOMIE_UNIT_WATT,
};

use crate::SMARTHOME_TYPE_NUMERIC;

pub const NUMERIC_NODE_DEFAULT_ID: &str = "numeric";
pub const NUMERIC_NODE_DEFAULT_NAME: &str = "Numeric Sensor";
pub const NUMERIC_NODE_VALUE_PROP_ID: &str = "value";

pub enum NumericSensorType {
    Generic,
    Temperature,
    Humidity,
    Pressure,
    Volume,
    Volt,
    Current,
    Power,
    Energy,
    Frequency,
    Battery,
    Distance,
    Speed,
    Light,
    GasCO,
    GasCO2,
    GasCH4,
    GasVoc,
}

impl NumericSensorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NumericSensorType::Generic => "generic",
            NumericSensorType::Temperature => "temperature",
            NumericSensorType::Humidity => "humidity",
            NumericSensorType::Pressure => "pressure",
            NumericSensorType::Volume => "volume",
            NumericSensorType::Volt => "volt",
            NumericSensorType::Current => "current",
            NumericSensorType::Power => "power",
            NumericSensorType::Energy => "energy",
            NumericSensorType::Frequency => "frequency",
            NumericSensorType::Battery => "battery",
            NumericSensorType::Distance => "distance",
            NumericSensorType::Speed => "speed",
            NumericSensorType::Light => "light",
            NumericSensorType::GasCO => "gas-co",
            NumericSensorType::GasCO2 => "gas-co2",
            NumericSensorType::GasCH4 => "gas-ch4",
            NumericSensorType::GasVoc => "gas-voc",
        }
    }

    pub fn make_smarthome_type(&self) -> String {
        format!("{}-{}", SMARTHOME_TYPE_NUMERIC, self.as_str())
    }

    pub fn default_node_name(&self) -> &'static str {
        if let NumericSensorType::Generic = self {
            "numeric-sensor"
        } else {
            self.as_str()
        }
    }
}

#[derive(Debug)]
pub struct NumericSensorNode {
    pub publisher: NumericSensorNodePublisher,
    pub value: f64,
}

pub struct NumericSensorNodeBuilder {
    node_builder: NodeDescriptionBuilder,
}

impl Default for NumericSensorNodeBuilder {
    fn default() -> Self {
        NumericSensorNodeBuilder::for_type(NumericSensorType::Generic)
    }
}

impl NumericSensorNodeBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn for_type(sensor_type: NumericSensorType) -> Self {
        let db = Self::build_node(
            NodeDescriptionBuilder::new().name(sensor_type.default_node_name()),
            &sensor_type,
        )
        .r#type(sensor_type.make_smarthome_type());

        Self { node_builder: db }
    }

    fn build_node(
        db: NodeDescriptionBuilder,
        sensor_type: &NumericSensorType,
    ) -> NodeDescriptionBuilder {
        let prop_id = NUMERIC_NODE_VALUE_PROP_ID.try_into().unwrap();
        let prop_name = format!("Sensor value ({})", sensor_type.as_str());
        let mut propbuilder = PropertyDescriptionBuilder::new(homie5::HomieDataType::Integer)
            .name(prop_name)
            .settable(false)
            .retained(true);

        propbuilder = match sensor_type {
            NumericSensorType::Generic => propbuilder,
            NumericSensorType::Temperature => propbuilder
                .unit(HOMIE_UNIT_DEGREE_CELSIUS)
                .datatype(homie5::HomieDataType::Float)
                .format(HomiePropertyFormat::FloatRange(FloatRange {
                    min: Some(-273.15),
                    max: None,
                    step: None,
                })),
            NumericSensorType::Humidity => propbuilder
                .unit(HOMIE_UNIT_PERCENT)
                .datatype(homie5::HomieDataType::Float)
                .format(HomiePropertyFormat::FloatRange(FloatRange {
                    min: Some(0.0),
                    max: Some(100.0),
                    step: None,
                })),
            NumericSensorType::Pressure => propbuilder
                .unit(HOMIE_UNIT_KILOPASCAL)
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::Volume => propbuilder
                .unit(HOMIE_UNIT_LITER)
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::Volt => propbuilder
                .unit(HOMIE_UNIT_VOLT)
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::Current => propbuilder
                .unit(HOMIE_UNIT_AMPERE)
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::Power => propbuilder
                .unit(HOMIE_UNIT_WATT)
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::Energy => propbuilder
                .unit(HOMIE_UNIT_KILOWATTHOUR)
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::Frequency => propbuilder
                .unit(HOMIE_UNIT_HERTZ)
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::Battery => propbuilder
                .unit(HOMIE_UNIT_PERCENT)
                .datatype(homie5::HomieDataType::Float)
                .format(HomiePropertyFormat::FloatRange(FloatRange {
                    min: Some(0.0),
                    max: Some(100.0),
                    step: None,
                })),
            NumericSensorType::Distance => propbuilder
                .unit(HOMIE_UNIT_METER)
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::Speed => propbuilder
                .unit("m/s")
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::Light => propbuilder
                .unit(HOMIE_UNIT_LUX)
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::GasCO => propbuilder
                .unit("ppm")
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::GasCO2 => propbuilder
                .unit("ppm")
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::GasCH4 => propbuilder
                .unit("ppm")
                .datatype(homie5::HomieDataType::Float),
            NumericSensorType::GasVoc => propbuilder
                .unit("ppm")
                .datatype(homie5::HomieDataType::Float),
        };

        db.add_property(prop_id, propbuilder.build())
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
    ) -> (HomieNodeDescription, NumericSensorNodePublisher) {
        (
            self.node_builder.build(),
            NumericSensorNodePublisher::new(
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

#[derive(Debug)]
pub struct NumericSensorNodePublisher {
    client: Homie5DeviceProtocol,
    node: NodeRef,
    value_prop: HomieID,
}

impl NumericSensorNodePublisher {
    pub fn new(node: NodeRef, client: Homie5DeviceProtocol) -> Self {
        Self {
            node,
            client,
            value_prop: NUMERIC_NODE_VALUE_PROP_ID.try_into().unwrap(),
        }
    }

    pub fn value(&self, value: f64) -> homie5::client::Publish {
        self.client.publish_value(
            self.node.node_id(),
            &self.value_prop,
            value.to_string(),
            true,
        )
    }
}

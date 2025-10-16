use beetry_backend::ConditionBehavior;
use beetry_definitions::{
    description::{LeafDescription, LeafKind, NodeHashProvider},
    parameter::{self, Bounds, ProvideSchema, Schema, SerializedParametersMarker},
};
use beetry_macros::ProvideSchema;
use beetry_plugin::node::{self, ConditionFactory, NodePlugin};
use serde::{Deserialize, Serialize};
use tracing::debug;
use type_hash::TypeHash;

#[derive(TypeHash)]
pub struct CheckBattery {
    params: CheckBatteryParams,
}

#[derive(Serialize, Deserialize, ProvideSchema, TypeHash)]
pub struct CheckBatteryParams {
    #[param(
        description = "charged battery level in percentage",
        min = 0,
        max = 100
    )]
    level: f32,
}

impl SerializedParametersMarker for CheckBatteryParams {}

impl Default for CheckBatteryParams {
    fn default() -> Self {
        Self { level: 100.0 }
    }
}

impl CheckBattery {
    pub fn new(params: CheckBatteryParams) -> Self {
        Self { params }
    }

    pub fn check(&mut self) -> bool {
        self.params.level -= 1.0;
        let result = self.params.level > 95.0;
        debug!("level: {}, above threshold? {result}", self.params.level);
        result
    }
}

impl ConditionBehavior for CheckBattery {
    fn cond(&mut self) -> bool {
        self.check()
    }

    fn reset(&mut self) {
        *self = Self::new(CheckBatteryParams::default());
    }
}

pub struct CheckBatteryPlugin {
    factory: node::ConditionFactory,
}

impl NodePlugin for CheckBatteryPlugin {
    type Description = LeafDescription;
    type Factory = ConditionFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            factory: node::ConditionFactory::new(Box::new(|reconstruct| {
                Ok(
                    Box::new(CheckBattery::new(reconstruct.parameters.try_into()?))
                        as Box<dyn ConditionBehavior>,
                )
            })),
        }
    }

    fn desc(&self) -> LeafDescription {
        LeafDescription::builder()
            .name("CheckBattery")
            .hash(CheckBattery::hash())
            .kind(LeafKind::Condition)
            .params_schema(CheckBatteryParams::provide())
            .build()
    }

    fn factory(self: Box<Self>) -> node::ConditionFactory {
        self.factory
    }
}

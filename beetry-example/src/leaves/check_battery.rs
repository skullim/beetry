use beetry_core::{BoxConditionBehavior, ConditionBehavior};
use beetry_macros::ProvideSchema;
use beetry_plugin::Plugin;
use beetry_plugin::node::{self, ConditionFactory};
use beetry_serde::de::parameter::ParametersMarker;
use beetry_serde::ser::node::{ActionLeafSchema, ActionSpec, LeafSpec, NodeName};
use beetry_serde::ser::parameter::{self, Bounds, ProvideSchema, Schema};
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

impl ParametersMarker for CheckBatteryParams {}

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

impl Plugin for CheckBatteryPlugin {
    type Spec = LeafSpec;
    type Factory = ConditionFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            factory: ConditionFactory::new(Box::new(|data| {
                Ok(Box::new(CheckBattery::new(data.parameters.try_into()?))
                    as BoxConditionBehavior)
            })),
        }
    }

    fn spec(&self) -> ActionSpec {
        ActionSpec::builder()
            .name(NodeName::new("CheckBattery"))
            .schema(ActionLeafSchema::default())
            .params_schema(CheckBatteryParams::provide())
            .build()
    }

    fn factory(self: Box<Self>) -> ConditionFactory {
        self.factory
    }
}

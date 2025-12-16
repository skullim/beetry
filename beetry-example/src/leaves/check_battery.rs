use beetry_core::{BoxConditionBehavior, ConditionBehavior};
use beetry_macros::ProvideSchema;
use beetry_plugin::{node::ConditionReconstructionData, plugin};

use beetry_plugin_types::{
    parameter::{self, Bounds, ProvideSchema, Schema},
    spec,
};
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

plugin! {
  CheckBatteryPlugin: Condition {
    spec = spec! {type = condition, name = "CheckBattery", params = CheckBatteryParams::provide()},
    factory_fn = |data: ConditionReconstructionData| {
            Ok(
                Box::new(
                    CheckBattery::new(beetry_reconstruction_types::parameter::Deserializer::deserialize(data.parameters)?))
                    as BoxConditionBehavior,
            )
    }
  }
}

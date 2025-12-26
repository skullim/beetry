use beetry_core::{BoxConditionBehavior, ConditionBehavior};
use beetry_editor_types::spec::node::{
    self, Bounds, Definition, NodeKind, NodeName, NodeSpec, NodeSpecKey, ProvideSchema, Schema,
};
use beetry_macros::ProvideSchema;
use beetry_plugin::{
    Plugin,
    node::{ConditionFactory, ConditionReconstructionData},
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

// plugin! {
//   CheckBatteryPlugin: Condition {
//     spec = spec! {type = condition, name = "CheckBattery", params = CheckBatteryParams::provide()},
//     factory_fn = |data: ConditionReconstructionData| {
//             Ok(
//                 Box::new(
//                     CheckBattery::new(beetry_reconstruction_types::parameter::Deserializer::deserialize(data.parameters)?))
//                     as BoxConditionBehavior,
//             )
//     }
//   }
// }

pub struct CheckBatteryPlugin {
    spec: NodeSpec,
    factory: ConditionFactory,
}

impl Plugin for CheckBatteryPlugin {
    type Spec = NodeSpec;
    type Factory = ConditionFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        let factory_fn = |data: ConditionReconstructionData| {
            Ok(Box::new(CheckBattery::new(
                beetry_editor_types::output::node::Deserializer::deserialize(data.parameters)?,
            )) as BoxConditionBehavior)
        };
        let spec = NodeSpec::builder()
            .key(NodeSpecKey::new(
                NodeName::new("Check Battery"),
                NodeKind::condition(),
            ))
            .params(CheckBatteryParams::provide())
            .build();

        Self {
            spec,
            factory: Self::Factory::new(Box::new(factory_fn)),
        }
    }

    fn spec(&self) -> &Self::Spec {
        &self.spec
    }

    fn factory(&self) -> &Self::Factory {
        &self.factory
    }

    fn into_parts(self: Box<Self>) -> (Self::Spec, Self::Factory) {
        (self.spec, self.factory)
    }
}

use beetry_core::ConditionBehavior;
use beetry_macros::ProvideSchema;
use beetry_plugin::node::{self, ConditionFactory, NodePlugin};
use beetry_serde::de::parameter::ParametersMarker;
use beetry_serde::ser::node::{LeafKind, LeafSchema, LeafSpec, NodeName};
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

impl NodePlugin for CheckBatteryPlugin {
    type Spec = LeafSpec;
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

    fn spec(&self) -> LeafSpec {
        LeafSpec::new(
            NodeName::new("CheckBattery"),
            LeafSchema::builder()
                .kind(LeafKind::Condition)
                .params(CheckBatteryParams::provide())
                .build(),
        )
    }

    fn factory(self: Box<Self>) -> node::ConditionFactory {
        self.factory
    }
}

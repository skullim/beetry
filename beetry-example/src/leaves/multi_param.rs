use beetry_core::ConditionBehavior;
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
pub struct MultiParamCond {
    params: MultiParamCondParams,
}

#[derive(Serialize, Deserialize, ProvideSchema, TypeHash)]
pub struct MultiParamCondParams {
    #[param(
        description = "charged battery level in percentage",
        min = 0,
        max = 100
    )]
    level: f32,
}

#[derive(Serialize, Deserialize)]
pub struct DurationParam;

impl SerializedParametersMarker for MultiParamCondParams {}

impl Default for MultiParamCondParams {
    fn default() -> Self {
        Self { level: 100.0 }
    }
}

impl MultiParamCond {
    pub fn new(params: MultiParamCondParams) -> Self {
        Self { params }
    }

    pub fn check(&mut self) -> bool {
        self.params.level -= 1.0;
        let result = self.params.level > 95.0;
        debug!("level: {}, above threshold? {result}", self.params.level);
        result
    }
}

impl ConditionBehavior for MultiParamCond {
    fn cond(&mut self) -> bool {
        self.check()
    }

    fn reset(&mut self) {
        *self = Self::new(MultiParamCondParams::default());
    }
}

pub struct MultiParamCondPlugin {
    factory: node::ConditionFactory,
}

impl NodePlugin for MultiParamCondPlugin {
    type Description = LeafDescription;
    type Factory = ConditionFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            factory: node::ConditionFactory::new(Box::new(|reconstruct| {
                Ok(
                    Box::new(MultiParamCond::new(reconstruct.parameters.try_into()?))
                        as Box<dyn ConditionBehavior>,
                )
            })),
        }
    }

    fn desc(&self) -> LeafDescription {
        LeafDescription::builder()
            .name("MultiParamCond")
            .hash(MultiParamCond::hash())
            .kind(LeafKind::Condition)
            .params_schema(MultiParamCondParams::provide())
            .build()
    }

    fn factory(self: Box<Self>) -> node::ConditionFactory {
        self.factory
    }
}

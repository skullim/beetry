use anyhow::anyhow;
use beetry_core::ConditionBehavior;
use beetry_editor_types::spec::node::{
    FieldDefinition, FieldMetadata, FieldName, FieldTypeSpec, ParamsSpec, ProvideParamSpec,
};
use beetry_plugin::condition;
use mitsein::iter1::IntoIterator1;
use std::sync::Arc;

use beetry_reconstruction::ParamsReconstructor;
use serde::{Deserialize, Serialize};
use tracing::debug;
use type_hash::TypeHash;

#[derive(TypeHash)]
pub struct CheckBattery {
    params: CheckBatteryParams,
}

#[derive(Serialize, Deserialize, TypeHash)]
pub struct CheckBatteryParams {
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

impl ProvideParamSpec for CheckBatteryParams {
    fn provide() -> ParamsSpec {
        [(
            FieldName::from("level"),
            FieldDefinition {
                type_spec: FieldTypeSpec::F64(FieldMetadata::new(Arc::new(|level| {
                    if *level > 100.0 {
                        Err(anyhow!("level must be lower than 100%"))
                    } else if *level < 0.0 {
                        Err(anyhow!("level cannot be lower than 0%"))
                    } else {
                        Ok(())
                    }
                }))),
                description: Some("charged battery level in percentage".into()),
            },
        )]
        .into_iter1()
        .collect1()
    }
}

condition! {
    CheckBatteryPlugin: "Check Battery";
    params(parameters): CheckBatteryParams::provide();
    create: CheckBattery::new(ParamsReconstructor::reconstruct(parameters)?);
}

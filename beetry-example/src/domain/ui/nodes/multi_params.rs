use std::sync::Arc;

use anyhow::anyhow;
use beetry::{
    leaf::{ActionBehavior, NodeTask, Task},
    plugin::{
        FieldDefinition, FieldMetadata, FieldName, FieldTypeSpec, ParamsDeserializer, ParamsSpec,
        ProvideParamSpec, action,
    },
    runtime::TickStatus,
};
use mitsein::iter1::IntoIterator1;
use serde::{Deserialize, Serialize};
use tracing::debug;

pub struct MultiParams {
    params: MultiParamsParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiParamsParams {
    enabled: bool,
    retry_limit: u64,
    altitude_offset_m: i64,
    target_accuracy_m: f64,
    profile_name: String,
}

impl Default for MultiParamsParams {
    fn default() -> Self {
        Self {
            enabled: true,
            retry_limit: 3,
            altitude_offset_m: 0,
            target_accuracy_m: 0.5,
            profile_name: "default".to_string(),
        }
    }
}

impl MultiParams {
    pub fn new(params: MultiParamsParams) -> Self {
        Self { params }
    }
}

impl ActionBehavior for MultiParams {
    fn task(&mut self) -> anyhow::Result<NodeTask> {
        Ok(NodeTask::new(MultiParamsTask::new(self.params.clone())))
    }
}

struct MultiParamsTask {
    params: MultiParamsParams,
}

impl MultiParamsTask {
    fn new(params: MultiParamsParams) -> Self {
        Self { params }
    }
}

impl Task for MultiParamsTask {
    async fn run(self) -> TickStatus {
        debug!("applying multi params: {:?}", self.params);
        TickStatus::Success
    }
}

impl ProvideParamSpec for MultiParamsParams {
    fn provide() -> ParamsSpec {
        [
            (
                FieldName::from("enabled"),
                FieldDefinition {
                    type_spec: FieldTypeSpec::Bool(FieldMetadata::default()),
                    description: Some("Enable or disable mission execution".into()),
                },
            ),
            (
                FieldName::from("retry_limit"),
                FieldDefinition {
                    type_spec: FieldTypeSpec::U64(FieldMetadata::new(Arc::new(|value| {
                        if *value == 0 {
                            Err(anyhow!("retry_limit must be greater than 0"))
                        } else if *value > 20 {
                            Err(anyhow!("retry_limit must be at most 20"))
                        } else {
                            Ok(())
                        }
                    }))),
                    description: Some("Maximum number of retry attempts".into()),
                },
            ),
            (
                FieldName::from("altitude_offset_m"),
                FieldDefinition {
                    type_spec: FieldTypeSpec::I64(FieldMetadata::new(Arc::new(|value| {
                        if *value < -500 || *value > 500 {
                            Err(anyhow!("altitude_offset_m must be in range [-500, 500]"))
                        } else {
                            Ok(())
                        }
                    }))),
                    description: Some("Signed altitude offset in meters".into()),
                },
            ),
            (
                FieldName::from("target_accuracy_m"),
                FieldDefinition {
                    type_spec: FieldTypeSpec::F64(FieldMetadata::new(Arc::new(|value| {
                        if *value <= 0.0 {
                            Err(anyhow!("target_accuracy_m must be greater than 0"))
                        } else if *value > 10.0 {
                            Err(anyhow!("target_accuracy_m must be at most 10"))
                        } else {
                            Ok(())
                        }
                    }))),
                    description: Some("Desired localization accuracy in meters".into()),
                },
            ),
            (
                FieldName::from("profile_name"),
                FieldDefinition {
                    type_spec: FieldTypeSpec::String(FieldMetadata::new(Arc::new(|value| {
                        if value.trim().is_empty() {
                            Err(anyhow!("profile_name cannot be empty"))
                        } else {
                            Ok(())
                        }
                    }))),
                    description: Some("Configuration profile label".into()),
                },
            ),
        ]
        .into_iter1()
        .collect1()
    }
}

action! {
    MultiParamsPlugin: "Multi Params";
    params(parameters): MultiParamsParams::provide();
    create: MultiParams::new(ParamsDeserializer::deserialize(parameters)?);
}

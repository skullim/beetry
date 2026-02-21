use anyhow::anyhow;
use beetry_core::{ActionBehavior, NodeTask, Task, TickStatus};
use beetry_editor_types::spec::node::{
    FieldDefinition, FieldMetadata, FieldName, FieldTypeSpec, ParamsSpec, ProvideParamSpec,
};
use beetry_plugin::action;
use beetry_reconstruction::ParamsReconstructor;
use mitsein::iter1::IntoIterator1;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

pub struct MissionConfig {
    params: MissionConfigParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionConfigParams {
    enabled: bool,
    retry_limit: u64,
    altitude_offset_m: i64,
    target_accuracy_m: f64,
    profile_name: String,
}

impl Default for MissionConfigParams {
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

impl MissionConfig {
    pub fn new(params: MissionConfigParams) -> Self {
        Self { params }
    }
}

impl ActionBehavior for MissionConfig {
    fn task(&mut self) -> anyhow::Result<NodeTask> {
        Ok(NodeTask::new(MissionConfigTask::new(self.params.clone())))
    }
}

struct MissionConfigTask {
    params: MissionConfigParams,
}

impl MissionConfigTask {
    fn new(params: MissionConfigParams) -> Self {
        Self { params }
    }
}

impl Task for MissionConfigTask {
    async fn run(self) -> TickStatus {
        debug!("applying mission config: {:?}", self.params);
        TickStatus::Success
    }
}

impl ProvideParamSpec for MissionConfigParams {
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
    MissionConfigPlugin: "Mission Config";
    params(parameters): MissionConfigParams::provide();
    create: MissionConfig::new(ParamsReconstructor::reconstruct(parameters)?);
}

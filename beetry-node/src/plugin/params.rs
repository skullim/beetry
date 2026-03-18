use std::sync::Arc;

use anyhow::{Error, anyhow};
use beetry_editor_types::{
    output::node::Parameters,
    spec::node::{FieldDefinition, FieldMetadata, FieldName, FieldTypeSpec, ParamsSpec},
};
use beetry_plugin::{ProvideParamSpec, node::ParamsDeserializer};
use mitsein::iter1::IntoIterator1;

use crate::ParallelParams;

impl ProvideParamSpec for ParallelParams {
    fn provide() -> ParamsSpec {
        [
            (
                FieldName::from("success_count"),
                FieldDefinition {
                    type_spec: FieldTypeSpec::U16(FieldMetadata::new(Arc::new(
                        Self::validate_success_count,
                    ))),
                    description: Some("Number of successful children required to succeed".into()),
                },
            ),
            (
                FieldName::from("failure_count"),
                FieldDefinition {
                    type_spec: FieldTypeSpec::U16(FieldMetadata::new(Arc::new(
                        Self::validate_failure_count,
                    ))),
                    description: Some("Number of failed children required to fail".into()),
                },
            ),
        ]
        .into_iter1()
        .collect1()
    }
}

impl ParallelParams {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn validate_success_count(value: &u16) -> Result<(), Error> {
        if *value == 0 {
            return Err(anyhow!("success count must be greater than 0"));
        }
        Ok(())
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn validate_failure_count(value: &u16) -> Result<(), Error> {
        if *value == 0 {
            return Err(anyhow!("failure count must be greater than 0"));
        }
        Ok(())
    }

    pub(crate) fn reconstruct(parameters: Parameters) -> Result<Self, Error> {
        let params: Self = ParamsDeserializer::deserialize(parameters)
            .map_err(|err| anyhow!("failed to deserialize Parallel parameters: {err}"))?;
        Ok(params)
    }
}

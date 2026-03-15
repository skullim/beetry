use anyhow::{Result, anyhow};
use beetry_editor_types::{
    id::NodeId,
    output::node::{ParameterValue, Parameters},
    spec::node::FieldTypeSpec,
};

use crate::repository::ParamValuesRepository;

pub struct ParameterValueQueryView<'a> {
    pub(crate) repo: &'a ParamValuesRepository,
}

pub trait ParameterValueQuery {
    fn parameters(&self, id: NodeId) -> Result<&Parameters>;
}

impl<'a> ParameterValueQueryView<'a> {
    pub(crate) fn new(repo: &'a ParamValuesRepository) -> Self {
        Self { repo }
    }
}

impl ParameterValueQuery for ParameterValueQueryView<'_> {
    fn parameters(&self, id: NodeId) -> Result<&Parameters> {
        self.repo
            .params(id)
            .ok_or_else(|| anyhow!("failed to obtain parameters for node {id}"))
    }
}

pub struct ParameterValueViewMut<'a> {
    repo: &'a mut ParamValuesRepository,
}

pub trait ParameterValueMut {
    fn create(&mut self, id: NodeId, params: Parameters);
}

impl<'a> ParameterValueViewMut<'a> {
    pub(crate) fn new(repo: &'a mut ParamValuesRepository) -> Self {
        Self { repo }
    }

    pub fn create(&mut self, id: NodeId, params: Parameters) {
        self.repo.create(id, params);
    }
}

impl ParameterValueMut for ParameterValueViewMut<'_> {
    fn create(&mut self, id: NodeId, params: Parameters) {
        ParameterValueViewMut::create(self, id, params);
    }
}

pub struct ParameterValueParser;

impl ParameterValueParser {
    pub fn parse(type_spec: &FieldTypeSpec, raw_value: String) -> Result<ParameterValue> {
        Ok(match &type_spec {
            FieldTypeSpec::Bool(meta) => {
                let parsed = raw_value.parse()?;
                meta.validate(&parsed)?;
                ParameterValue::Bool(parsed)
            }
            FieldTypeSpec::F64(meta) => {
                let parsed = raw_value.parse()?;
                meta.validate(&parsed)?;
                ParameterValue::F64(parsed)
            }
            FieldTypeSpec::I64(meta) => {
                let parsed = raw_value.parse()?;
                meta.validate(&parsed)?;
                ParameterValue::I64(parsed)
            }
            FieldTypeSpec::U16(meta) => {
                let parsed: u16 = raw_value.parse()?;
                meta.validate(&parsed)?;
                ParameterValue::U16(parsed)
            }
            FieldTypeSpec::U64(meta) => {
                let parsed = raw_value.parse()?;
                meta.validate(&parsed)?;
                ParameterValue::U64(parsed)
            }
            FieldTypeSpec::String(meta) => {
                meta.validate(&raw_value)?;
                ParameterValue::String(raw_value)
            }
        })
    }
}

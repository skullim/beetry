use crate::repository::ParamValueRepositoryConcept;
use anyhow::{Result, anyhow};
use beetry_editor_types::{
    id::NodeId,
    output::node::{ParameterValue, Parameters},
    spec::node::FieldTypeSpec,
};

pub struct ParameterValueQueryView<'a, PVR> {
    pub(crate) repo: &'a PVR,
}

pub trait ParameterValueQuery {
    fn parameters(&self, id: NodeId) -> Result<&Parameters>;
}

impl<'a, PVR> ParameterValueQueryView<'a, PVR> {
    pub(crate) fn new(repo: &'a PVR) -> Self {
        Self { repo }
    }
}

impl<PVR> ParameterValueQuery for ParameterValueQueryView<'_, PVR>
where
    PVR: ParamValueRepositoryConcept,
{
    fn parameters(&self, id: NodeId) -> Result<&Parameters> {
        self.repo
            .params(id)
            .ok_or_else(|| anyhow!("failed to obtain parameters for node {id}"))
    }
}

pub struct ParameterValueViewMut<'a, PVR> {
    repo: &'a mut PVR,
}

pub trait ParameterValueMut {
    fn create(&mut self, id: NodeId, params: Parameters);
}

impl<'a, PVR> ParameterValueViewMut<'a, PVR>
where
    PVR: ParamValueRepositoryConcept,
{
    pub(crate) fn new(repo: &'a mut PVR) -> Self {
        Self { repo }
    }

    pub fn create(&mut self, id: NodeId, params: Parameters) {
        self.repo.create(id, params);
    }
}

impl<PVR> ParameterValueMut for ParameterValueViewMut<'_, PVR>
where
    PVR: ParamValueRepositoryConcept,
{
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

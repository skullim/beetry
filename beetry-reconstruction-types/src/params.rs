use anyhow::Result;
use beetry_editor_types::output::node::{ParameterValue, Parameters};
use serde::Deserialize;
use std::collections::BTreeMap;

pub struct ParamsReconstructor;

impl ParamsReconstructor {
    pub fn reconstruct<T>(params: Parameters) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let deserializer = serde_value::ValueDeserializer::<serde_value::DeserializerError>::new(
            serde_value::Value::Map(
                params
                    .into_iter()
                    .map(|(name, value)| {
                        let value = match value {
                            ParameterValue::Bool(b) => serde_value::Value::Bool(b),
                            ParameterValue::U16(u) => serde_value::Value::U16(u),
                            ParameterValue::U64(u) => serde_value::Value::U64(u),
                            ParameterValue::I64(i) => serde_value::Value::I64(i),
                            ParameterValue::F64(f) => serde_value::Value::F64(f),
                            ParameterValue::String(s) => serde_value::Value::String(s),
                        };
                        (serde_value::Value::String(name), value)
                    })
                    .collect::<BTreeMap<_, _>>(),
            ),
        );
        Ok(T::deserialize(deserializer)?)
    }
}

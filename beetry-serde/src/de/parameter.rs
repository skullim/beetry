use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_value::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parameters {
    value: Value,
}

impl Parameters {
    pub fn from_value(value: Value) -> Self {
        Self { value }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self { value: Value::Unit }
    }
}

pub struct Deserializer;

impl Deserializer {
    pub fn deserialize<T>(params: Parameters) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let deserializer =
            serde_value::ValueDeserializer::<serde_value::DeserializerError>::new(params.value);
        Ok(T::deserialize(deserializer)?)
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parameters {
    value: serde_json::Value,
}

impl Parameters {
    pub fn from_value(value: serde_json::Value) -> Self {
        Self { value }
    }
}

pub struct Deserializer;

impl Deserializer {
    pub fn deserialize<T>(params: Parameters) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        Ok(serde_json::from_value(params.value)?)
    }
}

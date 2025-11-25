use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_value::Value;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parameters {
    field_value_map: BTreeMap<String, Value>,
}

impl FromIterator<(String, Value)> for Parameters {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        Self {
            field_value_map: iter.into_iter().collect(),
        }
    }
}

pub struct Deserializer;

impl Deserializer {
    pub fn deserialize<T>(params: Parameters) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let deserializer = serde_value::ValueDeserializer::<serde_value::DeserializerError>::new(
            Value::Map(BTreeMap::from_iter(
                params
                    .field_value_map
                    .into_iter()
                    .map(|(k, v)| (Value::String(k), v)),
            )),
        );
        Ok(T::deserialize(deserializer)?)
    }
}

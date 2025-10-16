use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

pub trait Serializer {
    type Item: Serialize;
    fn serialize(value: &Self::Item) -> anyhow::Result<String>;
}

pub trait Deserializer<'a> {
    type Item: Deserialize<'a>;
    fn deserialize(raw: &'a str) -> anyhow::Result<Self::Item>;
}

pub struct YamlSerializer<T> {
    pd: PhantomData<T>,
}

impl<T: Serialize> Serializer for YamlSerializer<T> {
    type Item = T;
    fn serialize(value: &T) -> anyhow::Result<String> {
        Ok(serde_yml::to_string(value)?)
    }
}

pub struct YamlDeserializer<T> {
    pd: PhantomData<T>,
}

impl<'de, T> Deserializer<'de> for YamlDeserializer<T>
where
    T: Deserialize<'de>,
{
    type Item = T;
    fn deserialize(raw: &'de str) -> anyhow::Result<Self::Item> {
        Ok(serde_yml::from_str(raw)?)
    }
}

pub struct JsonSerializer<T> {
    pd: PhantomData<T>,
}

impl<T: Serialize> Serializer for JsonSerializer<T> {
    type Item = T;
    fn serialize(value: &T) -> anyhow::Result<String> {
        Ok(serde_json::to_string(value)?)
    }
}

pub struct JsonDeserializer<T> {
    pd: PhantomData<T>,
}

impl<'de, T> Deserializer<'de> for JsonDeserializer<T>
where
    T: Deserialize<'de>,
{
    type Item = T;
    fn deserialize(raw: &'de str) -> anyhow::Result<Self::Item> {
        Ok(serde_json::from_str(raw)?)
    }
}

pub struct BinarySerializer {
    config: bincode::config::Configuration,
}

impl Default for BinarySerializer {
    fn default() -> Self {
        Self {
            config: bincode::config::standard(),
        }
    }
}

impl BinarySerializer {
    pub fn new(config: bincode::config::Configuration) -> Self {
        Self { config }
    }

    pub fn serialize<E: bincode::Encode>(&self, value: &E) -> anyhow::Result<Vec<u8>> {
        Ok(bincode::encode_to_vec(value, self.config)?)
    }
}

pub struct BinaryDeserializer {
    config: bincode::config::Configuration,
}

impl Default for BinaryDeserializer {
    fn default() -> Self {
        Self {
            config: bincode::config::standard(),
        }
    }
}

impl BinaryDeserializer {
    pub fn new(config: bincode::config::Configuration) -> Self {
        Self { config }
    }

    pub fn decode<D: bincode::Decode<()>>(&self, src: &[u8]) -> anyhow::Result<D> {
        Ok(bincode::decode_from_slice(src, self.config)?.0)
    }
}

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

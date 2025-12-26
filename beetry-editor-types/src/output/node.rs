use std::collections::{BTreeMap, HashSet};

use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use serde_value::Value;
use tracing::warn;

use crate::id::ChannelId;

// Param Value

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parameters {
    field_value_map: BTreeMap<String, Value>,
}

impl Parameters {
    /// caller has to assure that new value is valid w.r.t. schema and validation logic
    pub fn update(&mut self, field: &str, new: Value) -> Result<()> {
        let old = self
            .field_value_map
            .get_mut(field)
            .ok_or_else(|| anyhow!("no field named: {field} found for given parameters"))?;
        *old = new;
        Ok(())
    }
}

impl FromIterator<(String, Value)> for Parameters {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        Self {
            field_value_map: iter.into_iter().collect(),
        }
    }
}

//@todo this should be moved somewhere else
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

// Port State

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodePortConnection {
    #[default]
    Unconnected,
    //@todo check if BTreeSet1 offers more convenient semantics
    Internal(HashSet<ChannelId>), // connections
    External,
}

impl NodePortConnection {
    pub fn is_external(&self) -> bool {
        matches!(self, Self::External)
    }

    pub fn is_valid(&self) -> bool {
        !matches!(self, Self::Unconnected)
    }

    pub fn connected(&self) -> impl Iterator<Item = &ChannelId> {
        if let Self::Internal(connected) = self {
            connected.iter()
        } else {
            std::collections::hash_set::Iter::default()
        }
    }

    pub fn connect(&mut self, id: ChannelId) -> Result<()> {
        match self {
            Self::Unconnected => *self = Self::Internal(<_>::from_iter(std::iter::once(id))),
            Self::Internal(connected) => {
                connected.insert(id);
            }
            Self::External => {
                warn!("attempted to connect {id} to external port, switching port to internal");
                *self = Self::Internal(<_>::from_iter(std::iter::once(id)));
            }
        }
        Ok(())
    }

    pub fn disconnect_all(&mut self) -> impl IntoIterator<Item = ChannelId> + use<> {
        if let Self::Internal(connected) = self {
            let connected = std::mem::take(connected);
            *self = Self::Unconnected;
            Some(connected)
        } else {
            None
        }
        .into_iter()
        .flatten()
    }

    pub fn disconnect(&mut self, id: ChannelId) -> Result<()> {
        match self {
            invalid @ (Self::Unconnected | Self::External) => {
                bail!("attempted to remove connection from {invalid:?}")
            }
            Self::Internal(connected) => {
                if !connected.remove(&id) {
                    bail!("attempted to remove connection to {id} which does not exist");
                }
                if connected.is_empty() {
                    *self = Self::Unconnected;
                }
                Ok(())
            }
        }
    }
}

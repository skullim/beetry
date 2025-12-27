use crate::id::ChannelId;
use anyhow::{Result, bail};
use mitsein::{
    btree_set1::BTreeSet1,
    iter1::{FromIterator1, IntoIterator1},
};
use serde::{Deserialize, Serialize};
use serde_value::Value;
use std::collections::BTreeMap;
use tracing::warn;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parameters {
    field_value_map: BTreeMap<String, Value>,
}

impl IntoIterator for Parameters {
    type Item = (String, Value);
    type IntoIter = std::collections::btree_map::IntoIter<String, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.field_value_map.into_iter()
    }
}

impl Parameters {
    /// caller has to assure that new value is valid w.r.t. schema and validation logic
    pub fn set(&mut self, field: &str, value: Value) {
        //@todo in case no value for given field
        match self.field_value_map.get_mut(field) {
            Some(old) => {
                *old = value;
            }
            None => {
                self.field_value_map.insert(field.into(), value);
            }
        }
    }

    pub fn value(&self, field: &str) -> Option<&Value> {
        self.field_value_map.get(field)
    }
}

impl FromIterator<(String, Value)> for Parameters {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        Self {
            field_value_map: iter.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InternalPortConnections {
    conns: BTreeSet1<ChannelId>,
}

impl FromIterator1<ChannelId> for InternalPortConnections {
    fn from_iter1<T: IntoIterator1<Item = ChannelId>>(iter: T) -> Self {
        Self {
            conns: iter.into_iter1().collect1(),
        }
    }
}

impl IntoIterator for InternalPortConnections {
    type IntoIter = std::collections::btree_set::IntoIter<ChannelId>;
    type Item = ChannelId;
    fn into_iter(self) -> Self::IntoIter {
        self.conns.into_iter()
    }
}

impl InternalPortConnections {
    pub fn iter(&self) -> impl Iterator<Item = &ChannelId> {
        self.conns.iter1().into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortConnectionState {
    Internal(InternalPortConnections),
    External,
}

impl PortConnectionState {
    pub fn is_external(&self) -> bool {
        matches!(self, Self::External)
    }

    pub fn connect(&mut self, id: ChannelId) -> Result<()> {
        match self {
            Self::Internal(connected) => {
                connected.conns.insert(id);
            }
            Self::External => {
                warn!("attempted to connect {id} to external port, switching port to internal");
                *self = Self::Internal(<_>::try_from_iter(std::iter::once(id))?);
            }
        }
        Ok(())
    }

    pub fn disconnect_all(self) -> impl IntoIterator<Item = ChannelId> + use<> {
        if let Self::Internal(connected) = self {
            Some(connected.conns.into_iter())
        } else {
            None
        }
        .into_iter()
        .flatten()
    }

    pub fn disconnect(self, id: ChannelId) -> Result<Option<Self>> {
        match self {
            Self::External => {
                bail!("attempted to remove connection to channel {id} from external connection")
            }
            Self::Internal(connected) => match connected.conns.try_retain(|c| *c != id).ok() {
                Some(conns) => Ok(Some(Self::Internal(InternalPortConnections { conns }))),
                None => Ok(None),
            },
        }
    }
}

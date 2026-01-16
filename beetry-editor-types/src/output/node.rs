use crate::{id::ChannelId, spec::node::FieldName};
use anyhow::{Result, bail};
use mitsein::{
    btree_set1::BTreeSet1,
    iter1::{FromIterator1, IntoIterator1},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    Bool(bool),
    U64(u64),
    I64(i64),
    F64(f64),
    String(String),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Parameters {
    map: BTreeMap<FieldName, ParameterValue>,
}

impl Parameters {
    pub fn insert(&mut self, name: FieldName, value: ParameterValue) {
        self.map.insert(name, value);
    }

    pub fn remove(&mut self, name: &FieldName) {
        self.map.remove(name);
    }

    pub fn get_mut(&mut self, name: &FieldName) -> Option<&mut ParameterValue> {
        self.map.get_mut(name)
    }

    pub fn get(&self, name: &FieldName) -> Option<&ParameterValue> {
        self.map.get(name)
    }
}

impl FromIterator<(FieldName, ParameterValue)> for Parameters {
    fn from_iter<T: IntoIterator<Item = (FieldName, ParameterValue)>>(iter: T) -> Self {
        Self {
            map: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for Parameters {
    type Item = (FieldName, ParameterValue);
    type IntoIter = std::collections::btree_map::IntoIter<FieldName, ParameterValue>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
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
                error!(
                    "attempted to connect {id} to external port, mark the port internal to allow it"
                );
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

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    id::{ChannelId, ChannelSpecId},
    output::channel::ChannelData,
    spec::channel::ChannelSpec,
};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Store {
    pub specs: SpecStore,
    pub channels: DataStore,
}

impl FromIterator<(ChannelSpecId, ChannelSpec)> for SpecStore {
    fn from_iter<T: IntoIterator<Item = (ChannelSpecId, ChannelSpec)>>(iter: T) -> Self {
        Self {
            store: iter.into_iter().collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SpecStore {
    store: BTreeMap<ChannelSpecId, ChannelSpec>,
}

impl SpecStore {
    pub fn get(&self, id: &ChannelSpecId) -> Option<&ChannelSpec> {
        self.store.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ChannelSpecId, &ChannelSpec)> {
        self.store.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &ChannelSpec> {
        self.store.values()
    }

    pub fn into_records(self) -> impl Iterator<Item = SpecRecord> {
        self.store
            .into_iter()
            .map(|(id, spec)| SpecRecord { id, spec })
    }
}

impl FromIterator<(ChannelId, ChannelData)> for DataStore {
    fn from_iter<T: IntoIterator<Item = (ChannelId, ChannelData)>>(iter: T) -> Self {
        Self {
            store: iter.into_iter().collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DataStore {
    store: BTreeMap<ChannelId, ChannelData>,
}

impl DataStore {
    pub fn get(&self, id: &ChannelId) -> Option<&ChannelData> {
        self.store.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ChannelId, &ChannelData)> {
        self.store.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &ChannelData> {
        self.store.values()
    }

    pub fn into_records(self) -> impl Iterator<Item = Record> {
        self.store.into_iter().map(|(id, data)| Record { id, data })
    }
}

pub struct SpecRecord {
    pub id: ChannelSpecId,
    pub spec: ChannelSpec,
}

pub struct Record {
    pub id: ChannelId,
    pub data: ChannelData,
}

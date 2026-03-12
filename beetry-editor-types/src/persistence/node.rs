use std::collections::BTreeMap;

use getset::{CopyGetters, Getters};
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

use crate::{
    id::{NodeId, NodeSpecId},
    spec::node::NodeSpecKey,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    pub specs: SpecStore,
    pub nodes: RecordStore,
}

// The remaining parts of spec are to be loaded by the appropriate plugin.
#[derive(Debug, Serialize, Deserialize)]
pub struct SpecStore {
    // BTreeMap in favor of HashMap to have nicely ordered entries.
    store: BTreeMap<NodeSpecId, NodeSpecKey>,
}

impl FromIterator<(NodeSpecId, NodeSpecKey)> for SpecStore {
    fn from_iter<T: IntoIterator<Item = (NodeSpecId, NodeSpecKey)>>(iter: T) -> Self {
        Self {
            store: iter.into_iter().collect(),
        }
    }
}

impl SpecStore {
    pub fn get(&self, id: &NodeSpecId) -> Option<&NodeSpecKey> {
        self.store.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&NodeSpecId, &NodeSpecKey)> {
        self.store.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &NodeSpecKey> {
        self.store.values()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordStore {
    store: BTreeMap<NodeId, RecordValue>,
}

impl FromIterator<(NodeId, RecordValue)> for RecordStore {
    fn from_iter<T: IntoIterator<Item = (NodeId, RecordValue)>>(iter: T) -> Self {
        Self {
            store: iter.into_iter().collect(),
        }
    }
}

impl RecordStore {
    pub fn get(&self, id: &NodeId) -> Option<&RecordValue> {
        self.store.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = RecordView<'_>> {
        self.store
            .iter()
            .map(|(id, value)| RecordView { id, value })
    }

    pub fn values(&self) -> impl Iterator<Item = &RecordValue> {
        self.store.values()
    }

    pub fn into_records(self) -> impl Iterator<Item = Record> {
        self.store
            .into_iter()
            .map(|(id, value)| Record { id, value })
    }
}

pub struct Record {
    pub id: NodeId,
    pub value: RecordValue,
}

#[derive(Debug, Getters, CopyGetters)]
pub struct RecordView<'a> {
    #[getset(get_copy = "pub")]
    pub id: &'a NodeId,
    #[getset(get_copy = "pub")]
    pub value: &'a RecordValue,
}

#[derive(Debug, Getters, CopyGetters, Serialize, Deserialize)]
pub struct RecordValue {
    #[getset(get_copy = "pub")]
    spec_id: NodeSpecId,
    children: IndexSet<NodeId>,
}

impl RecordValue {
    pub fn new(spec_id: NodeSpecId, children: impl IntoIterator<Item = NodeId>) -> Self {
        Self {
            spec_id,
            children: children.into_iter().collect(),
        }
    }

    pub fn children(&self) -> impl Iterator<Item = &NodeId> {
        self.children.iter()
    }
}

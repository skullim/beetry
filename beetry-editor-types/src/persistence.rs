use std::collections::{BTreeMap, HashMap};

use crate::{
    id::{ChannelId, ChannelSpecId, NodeId, NodePortId, NodeSpecId},
    output::{
        channel::ChannelData,
        node::{Parameters, PortConnectionState},
        ui::{ChannelUiData, NodeUiData},
    },
    spec::{channel::ChannelSpec, node::NodeSpecKey},
};
use getset::{CopyGetters, Getters};
use indexmap::IndexSet;
use mitsein::{iter1::FromIterator1, vec1::Vec1};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EditorStateStore {
    pub tree: MaybeValidTree,
    pub ui_elements: UiElementStore,
}

// Editor might export/import either valid or (still) invalid tree
#[derive(Debug, Serialize, Deserialize)]
pub struct MaybeValidTree(pub TreeStore);

impl From<ValidTree> for MaybeValidTree {
    fn from(value: ValidTree) -> Self {
        Self(value.into_inner())
    }
}

// Proxy object to store valid tree
#[derive(Debug, Deserialize)]
pub struct ValidTree(TreeStore);

impl ValidTree {
    //@todo this should be hidden and only used in beetry-editor
    pub fn new(tree: TreeStore) -> Self {
        Self(tree)
    }

    pub fn into_inner(self) -> TreeStore {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TreeStore {
    pub node: NodeStore,
    pub ports: PortStateStore,
    pub parameter: ParameterStore,
    pub channel: ChannelStore,
}

impl TreeStore {
    pub fn new(
        node: NodeStore,
        ports: PortStateStore,
        parameter: ParameterStore,
        channel: ChannelStore,
    ) -> Self {
        Self {
            node,
            ports,
            parameter,
            channel,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeStore {
    pub specs: NodeSpecStore,
    pub nodes: NodeRecordStore,
}

//The remaining parts of spec are to be loaded by the appropriate plugin
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeSpecStore {
    // BTreeMap in favor of HashMap to have nicely ordered entries
    store: BTreeMap<NodeSpecId, NodeSpecKey>,
}

impl FromIterator<(NodeSpecId, NodeSpecKey)> for NodeSpecStore {
    fn from_iter<T: IntoIterator<Item = (NodeSpecId, NodeSpecKey)>>(iter: T) -> Self {
        Self {
            store: iter.into_iter().collect(),
        }
    }
}

impl NodeSpecStore {
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
pub struct NodeRecordStore {
    store: BTreeMap<NodeId, NodeRecordValue>,
}

impl FromIterator<(NodeId, NodeRecordValue)> for NodeRecordStore {
    fn from_iter<T: IntoIterator<Item = (NodeId, NodeRecordValue)>>(iter: T) -> Self {
        Self {
            store: iter.into_iter().collect(),
        }
    }
}

impl NodeRecordStore {
    pub fn get(&self, id: &NodeId) -> Option<&NodeRecordValue> {
        self.store.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = NodeRecordView<'_>> {
        self.store
            .iter()
            .map(|(id, value)| NodeRecordView { id, value })
    }

    pub fn values(&self) -> impl Iterator<Item = &NodeRecordValue> {
        self.store.values()
    }

    pub fn into_records(self) -> impl Iterator<Item = NodeRecord> {
        self.store
            .into_iter()
            .map(|(id, value)| NodeRecord { id, value })
    }
}

pub struct NodeRecord {
    pub id: NodeId,
    pub value: NodeRecordValue,
}

#[derive(Debug, Getters, CopyGetters)]
pub struct NodeRecordView<'a> {
    #[getset(get_copy = "pub")]
    pub id: &'a NodeId,
    #[getset(get_copy = "pub")]
    pub value: &'a NodeRecordValue,
}

#[derive(Debug, Getters, CopyGetters, Serialize, Deserialize)]
pub struct NodeRecordValue {
    #[getset(get_copy = "pub")]
    spec_id: NodeSpecId,
    children: IndexSet<NodeId>,
}

impl NodeRecordValue {
    pub fn children(&self) -> impl Iterator<Item = &NodeId> {
        self.children.iter()
    }
}

impl NodeRecordValue {
    pub fn new(spec_id: NodeSpecId, children: impl IntoIterator<Item = NodeId>) -> Self {
        Self {
            spec_id,
            children: children.into_iter().collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParameterStore {
    parameters: HashMap<NodeId, ParameterValues>,
}

impl ParameterStore {
    pub fn new(parameters: impl IntoIterator<Item = (NodeId, ParameterValues)>) -> Self {
        Self {
            parameters: parameters.into_iter().collect(),
        }
    }

    pub fn take(&mut self, id: &NodeId) -> Option<ParameterValues> {
        self.parameters.remove(id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParameterValues {
    pub params: Parameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortStateStore {
    ports: BTreeMap<NodeId, PortConnectionCollection>,
}

impl FromIterator<(NodeId, PortConnectionCollection)> for PortStateStore {
    fn from_iter<T: IntoIterator<Item = (NodeId, PortConnectionCollection)>>(iter: T) -> Self {
        Self {
            ports: iter.into_iter().collect(),
        }
    }
}

impl PortStateStore {
    pub fn take(&mut self, id: &NodeId) -> Option<PortConnectionCollection> {
        self.ports.remove(id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConnectionCollection {
    pub conns: Vec1<PortConnectionRecord>,
}

impl FromIterator1<PortConnectionRecord> for PortConnectionCollection {
    fn from_iter1<I>(items: I) -> Self
    where
        I: mitsein::prelude::IntoIterator1<Item = PortConnectionRecord>,
    {
        Self {
            conns: items.into_iter1().collect1(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConnectionRecord {
    pub port_id: NodePortId,
    pub conn: PortConnectionState,
}

impl PortConnectionRecord {
    pub fn new(port_id: NodePortId, conn: PortConnectionState) -> Self {
        Self { port_id, conn }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelStore {
    pub specs: ChannelSpecStore,
    pub channels: ChannelDataStore,
}

impl FromIterator<(ChannelSpecId, ChannelSpec)> for ChannelSpecStore {
    fn from_iter<T: IntoIterator<Item = (ChannelSpecId, ChannelSpec)>>(iter: T) -> Self {
        Self {
            store: iter.into_iter().collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelSpecStore {
    store: BTreeMap<ChannelSpecId, ChannelSpec>,
}

impl ChannelSpecStore {
    pub fn get(&self, id: &ChannelSpecId) -> Option<&ChannelSpec> {
        self.store.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ChannelSpecId, &ChannelSpec)> {
        self.store.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &ChannelSpec> {
        self.store.values()
    }

    pub fn into_records(self) -> impl Iterator<Item = ChannelSpecRecord> {
        self.store
            .into_iter()
            .map(|(id, spec)| ChannelSpecRecord { id, spec })
    }
}

impl FromIterator<(ChannelId, ChannelData)> for ChannelDataStore {
    fn from_iter<T: IntoIterator<Item = (ChannelId, ChannelData)>>(iter: T) -> Self {
        Self {
            store: iter.into_iter().collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelDataStore {
    store: BTreeMap<ChannelId, ChannelData>,
}

impl ChannelDataStore {
    pub fn get(&self, id: &ChannelId) -> Option<&ChannelData> {
        self.store.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ChannelId, &ChannelData)> {
        self.store.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &ChannelData> {
        self.store.values()
    }

    pub fn into_records(self) -> impl Iterator<Item = ChannelRecord> {
        self.store
            .into_iter()
            .map(|(id, data)| ChannelRecord { id, data })
    }
}

pub struct ChannelSpecRecord {
    pub id: ChannelSpecId,
    pub spec: ChannelSpec,
}

pub struct ChannelRecord {
    pub id: ChannelId,
    pub data: ChannelData,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UiElementStore {
    pub nodes: Vec<NodeUiRecord>,
    pub channels: Vec<ChannelUiRecord>,
}

impl UiElementStore {
    pub fn new(
        nodes: impl IntoIterator<Item = NodeUiRecord>,
        channels: impl IntoIterator<Item = ChannelUiRecord>,
    ) -> Self {
        Self {
            nodes: nodes.into_iter().collect(),
            channels: channels.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeUiRecord {
    pub id: NodeId,
    pub data: NodeUiData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelUiRecord {
    pub id: ChannelId,
    pub data: ChannelUiData,
}

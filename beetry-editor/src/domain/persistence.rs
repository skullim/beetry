use std::collections::HashMap;

use crate::domain::models::{
    ChannelData, ChannelSpecId, ChannelUiData, NodeId, NodePortConnection, NodePortId, NodeSpecId,
    NodeSpecKey, NodeUiData,
};
use beetry_plugin_types::channel::ChannelSpec;
use beetry_reconstruction_types::{channel::ChannelId, parameter};
use indexmap::IndexSet;

pub struct EditorStateStore {
    pub tree: MaybeValidTree,
    pub ui_elements: UiElementStore,
}

// Editor might export/import either valid or (still) invalid tree
pub struct MaybeValidTree(pub TreeStore);

impl From<ValidTree> for MaybeValidTree {
    fn from(value: ValidTree) -> Self {
        Self(value.into_inner())
    }
}

// Proxy object to store valid tree
pub struct ValidTree(TreeStore);

impl ValidTree {
    pub(crate) fn new(tree: TreeStore) -> Self {
        Self(tree)
    }

    pub fn into_inner(self) -> TreeStore {
        self.0
    }
}

pub struct TreeStore {
    pub node: NodeStore,
    pub ports: NodePortStore,
    pub parameter: ParameterValueStore,
    pub channel: ChannelStore,
}

impl TreeStore {
    pub fn new(
        node: NodeStore,
        ports: NodePortStore,
        parameter: ParameterValueStore,
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

pub struct NodeStore {
    pub specs: Vec<NodeSpecRecord>,
    pub nodes: Vec<NodeRecord>,
}

//The remaining parts of spec are to be loaded by the appropriate plugin
pub struct NodeSpecRecord {
    pub id: NodeSpecId,
    pub key: NodeSpecKey,
}

pub struct NodeRecord {
    pub id: NodeId,
    pub spec_id: NodeSpecId,
    pub children: IndexSet<NodeId>,
}

impl NodeRecord {
    pub fn new(id: NodeId, spec_id: NodeSpecId, children: impl Iterator<Item = NodeId>) -> Self {
        Self {
            id,
            spec_id,
            children: children.into_iter().collect(),
        }
    }
}

pub struct ParameterValueStore {
    parameters: HashMap<NodeId, ParameterValue>,
}

impl ParameterValueStore {
    pub fn new(parameters: impl IntoIterator<Item = (NodeId, ParameterValue)>) -> Self {
        Self {
            parameters: parameters.into_iter().collect(),
        }
    }

    pub fn take(&mut self, id: &NodeId) -> Option<ParameterValue> {
        self.parameters.remove(id)
    }
}

pub struct ParameterValue {
    pub params: parameter::Parameters,
}

pub struct NodePortStore {
    ports: HashMap<NodeId, NodePortState>,
}

impl NodePortStore {
    pub fn new(iter: impl IntoIterator<Item = (NodeId, NodePortState)>) -> Self {
        Self {
            ports: iter.into_iter().collect(),
        }
    }

    pub fn take(&mut self, id: &NodeId) -> Option<NodePortState> {
        self.ports.remove(id)
    }
}

pub struct NodePortState {
    pub conns: Vec<(NodePortId, NodePortConnection)>,
}

impl NodePortState {
    pub fn new(conns: impl IntoIterator<Item = (NodePortId, NodePortConnection)>) -> Self {
        Self {
            conns: conns.into_iter().collect(),
        }
    }
}

pub struct ChannelStore {
    pub specs: Vec<ChannelSpecRecord>,
    pub channels: Vec<ChannelRecord>,
}

pub struct ChannelSpecRecord {
    pub id: ChannelSpecId,
    pub spec: ChannelSpec,
}

pub struct ChannelRecord {
    pub id: ChannelId,
    pub data: ChannelData,
}

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Clone)]
pub struct NodeUiRecord {
    pub id: NodeId,
    pub data: NodeUiData,
}

#[derive(Debug, Clone)]
pub struct ChannelUiRecord {
    pub id: ChannelId,
    pub data: ChannelUiData,
}

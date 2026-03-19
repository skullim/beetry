//! This module models a tree as node snapshots derived from persisted editor
//! data.
//! It restores the tree hierarchy and produces a nested snapshot
//! representation that can later be consumed to build the runtime tree.

use std::collections::BTreeMap;

use anyhow::{Context, Result, anyhow, bail};
use beetry_editor_types::{
    id::{ChannelId, NodeId, NodePortId},
    output::node::Parameters,
    persistence,
    spec::node::{LeafKind, NodeKind, NodeName, NodePortKind, NodeSpecKey, PortKey},
};
use beetry_message::MessageSpec;
use bon::Builder;
use derive_more::From;
use getset::{CopyGetters, Getters};
use itertools::Itertools;
use mitsein::{iter1::FromIterator1, vec1::Vec1};

use crate::reconstruct::NodePluginRegistry;

#[derive(Debug, Clone)]
pub struct Root {
    pub child: Node,
}

impl Root {
    pub fn new(child: Node) -> Self {
        Self { child }
    }
}

#[derive(Debug, Clone, Builder)]
pub struct Node {
    #[builder(into)]
    pub name: NodeName,
    #[builder(into)]
    pub data: NodeData,
    #[builder(default)]
    pub parameters: Parameters,
}

impl Node {
    pub fn take_parameters(&mut self) -> Parameters {
        std::mem::take(&mut self.parameters)
    }
}

#[derive(Debug, Clone, From)]
pub enum NodeData {
    Control(Control),
    Decorator(Decorator),
    Leaf(Leaf),
}

#[derive(Debug, Clone, Getters)]
pub struct Control {
    children: Vec1<Node>,
}

impl Control {
    pub fn new(children: impl IntoIterator<Item = Node>) -> Result<Self> {
        let children = Vec1::try_from_iter(children)
            .map_err(|_| anyhow!("received empty children iterator"))?;
        Ok(Self { children })
    }

    pub fn into_children(self) -> impl IntoIterator<Item = Node> {
        self.children.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct Decorator {
    child: Box<Node>,
}

impl Decorator {
    pub fn new(child: Node) -> Self {
        Self {
            child: Box::new(child),
        }
    }

    pub fn into_child(self) -> Node {
        *self.child
    }
}

#[derive(Debug, Clone, Builder, Getters, CopyGetters)]
pub struct Leaf {
    #[get_copy = "pub"]
    kind: LeafKind,
    #[builder(default)]
    receivers: BTreeMap<PortKey, ChannelId>,
    #[builder(default)]
    senders: BTreeMap<PortKey, ChannelId>,
}

impl Leaf {
    pub fn take_receivers(&mut self) -> impl IntoIterator<Item = (PortKey, ChannelId)> {
        std::mem::take(&mut self.receivers)
    }

    pub fn take_senders(&mut self) -> impl IntoIterator<Item = (PortKey, ChannelId)> {
        std::mem::take(&mut self.senders)
    }
}

pub struct TreeSnapshotBuilder<'a> {
    node_store: persistence::node::Store,
    param_store: persistence::parameter::Store,
    port_store: persistence::port::Store,
    plugins: &'a NodePluginRegistry,
}

impl<'a> TreeSnapshotBuilder<'a> {
    pub fn new(
        node_store: persistence::node::Store,
        param_store: persistence::parameter::Store,
        port_store: persistence::port::Store,
        plugins: &'a NodePluginRegistry,
    ) -> Self {
        Self {
            node_store,
            param_store,
            port_store,
            plugins,
        }
    }

    pub fn build_root(&mut self) -> Result<Root> {
        let root_id = self
            .node_store
            .nodes
            .iter()
            .find_map(|record| {
                (self
                    .node_store
                    .specs
                    .get(&record.value.spec_id())
                    .map(NodeSpecKey::kind)
                    == Some(NodeKind::Root))
                .then_some(record.id)
            })
            .with_context(|| anyhow!("failed to find root id"))?;

        let root_node = self.node_store.nodes.get(root_id).with_context(|| {
            anyhow!("root node with id {root_id:?} does not exist in node store")
        })?;

        let root_child = *root_node
            .children()
            .next()
            .with_context(|| anyhow!("root node {root_id:?} does not have a child node"))?;

        Ok(Root::new(self.build_node(root_child)?))
    }

    fn build_control<'b>(
        &mut self,
        node_id: NodeId,
        name: NodeName,
        children: impl Iterator<Item = &'b NodeId>,
    ) -> Result<Node> {
        let children: Vec<_> = children
            .map(|child_id| self.build_node(*child_id))
            .collect::<Result<_>>()?;

        let params = self
            .param_store
            .take(&node_id)
            .map(|value| value.params)
            .unwrap_or_default();

        Ok(Node::builder()
            .name(name)
            .data(NodeData::Control(Control::new(children)?))
            .parameters(params)
            .build())
    }

    fn build_decorator(
        &mut self,
        node_id: NodeId,
        name: NodeName,
        child_id: NodeId,
    ) -> Result<Node> {
        let child = self.build_node(child_id)?;

        let params = self
            .param_store
            .take(&node_id)
            .map(|value| value.params)
            .unwrap_or_default();

        Ok(Node::builder()
            .name(name)
            .data(NodeData::Decorator(Decorator::new(child)))
            .parameters(params)
            .build())
    }

    fn build_leaf(&mut self, node_id: NodeId, name: NodeName, leaf_kind: LeafKind) -> Result<Node> {
        let spec = match leaf_kind {
            LeafKind::Action => self.plugins.action.get(&name)?.spec(),
            LeafKind::Condition => self.plugins.condition.get(&name)?.spec(),
        };

        let mut receivers = BTreeMap::new();
        let mut senders = BTreeMap::new();

        for (port_id, _port_state) in self.port_store.take_state(&node_id).into_iter().flatten() {
            let port_spec = spec
                .ports()
                .as_ref()
                .ok_or_else(|| anyhow!("expected port specification for node {name}"))?
                .spec(port_id)?;
            let port_key = port_spec.key.clone();

            let channel_id = self.port_store.connections_iter().find_map(|conn| {
                (conn.node_id == node_id && conn.port_id == port_id).then_some(conn.channel_id)
            });
            match port_spec.kind {
                NodePortKind::Receiver => {
                    if let Some(channel_id) = channel_id {
                        receivers.insert(port_key, channel_id);
                    }
                }
                NodePortKind::Sender => {
                    if let Some(channel_id) = channel_id {
                        senders.insert(port_key, channel_id);
                    }
                }
            }
        }

        let params = self
            .param_store
            .take(&node_id)
            .map(|value| value.params)
            .unwrap_or_default();

        let leaf_snapshot = Leaf::builder()
            .kind(leaf_kind)
            .receivers(receivers)
            .senders(senders)
            .build();

        Ok(Node::builder()
            .name(name)
            .data(leaf_snapshot)
            .parameters(params)
            .build())
    }

    fn build_node(&mut self, node_id: NodeId) -> Result<Node> {
        let node_record =
            self.node_store.nodes.remove(&node_id).with_context(|| {
                anyhow!("node with id {node_id:?} does not exist in node store")
            })?;
        let spec_id = node_record.spec_id();
        let spec_key = self
            .node_store
            .specs
            .get(&spec_id)
            .with_context(|| anyhow!("node spec with id {spec_id} does not exist"))?;
        let kind = spec_key.kind();
        let name = spec_key.name().clone();

        match kind {
            NodeKind::Control => self.build_control(node_id, name, node_record.children()),
            NodeKind::Decorator => {
                let child_id = node_record
                    .children()
                    .exactly_one()
                    .map_err(|children_iter| {
                        anyhow!(
                            "expected exactly one child for decorator node {name}, got {}",
                            children_iter.count()
                        )
                    })?;
                self.build_decorator(node_id, name, *child_id)
            }
            NodeKind::Leaf(leaf_kind) => self.build_leaf(node_id, name, leaf_kind),
            NodeKind::Root => bail!("unexpected Root node found during tree traversal"),
        }
    }
}

// Types to support external ports in the future

#[allow(dead_code)]
#[derive(Default, Builder)]
pub struct ExternalContextInfo {
    pub receivers: Vec<ExternalEndpointInfo>,
    pub senders: Vec<ExternalEndpointInfo>,
}

#[allow(dead_code)]
pub struct ExternalEndpointInfo {
    pub id: NodeId,
    pub name: NodeName,
    pub ports: Vec<ExternalPortInfo>,
}

#[allow(dead_code)]
pub struct ExternalPortInfo {
    pub id: NodePortId,
    pub message: MessageSpec,
}

use std::collections::HashSet;

use anyhow::{Result, anyhow, bail};
use bon::Builder;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    id::{ChannelId, ChannelSpecId, NodeId, NodePortId},
    output::channel::ChannelConfig,
    spec::{message::MessageSpec, node::NodeName},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodePosition {
    pub origin: Point,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodePortKind {
    Sender,
    Receiver,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodePortSpec {
    pub kind: NodePortKind,
    pub msg_spec: MessageSpec,
}

pub type ChannelPosition = NodePosition;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgePosition {
    pub start: Point,
    pub end: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeEdge {
    pub from: NodeId,
    pub to: NodeId,
}

pub struct IndexedDragOffset {
    pub id: NodeId,
    pub offset: Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    Action,
    Condition,
    Control,
    Decorator,
    Root,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct PortsSpec {
    spec: Vec<NodePortSpec>,
    ids: Vec<NodePortId>,
}

impl PortsSpec {
    //@todo tedious to use, sometimes only senders or receivers are present
    // also order matters here, much better to define builder
    pub fn new(
        senders: impl IntoIterator<Item = MessageSpec>,
        receivers: impl IntoIterator<Item = MessageSpec>,
    ) -> Self {
        let spec: Vec<_> = senders
            .into_iter()
            .map(|spec| NodePortSpec {
                kind: NodePortKind::Sender,
                msg_spec: spec,
            })
            .chain(receivers.into_iter().map(|spec| NodePortSpec {
                kind: NodePortKind::Receiver,
                msg_spec: spec,
            }))
            .collect();
        let ids: Vec<_> = (0..spec.len())
            .map(|id| NodePortId::new(id.try_into().unwrap()))
            .collect();
        Self { spec, ids }
    }

    pub fn ids(&self) -> &[NodePortId] {
        &self.ids
    }

    pub fn sender_ids(&self) -> impl Iterator<Item = &NodePortId> {
        self.senders().map(|(id, _)| id)
    }

    pub fn receiver_ids(&self) -> impl Iterator<Item = &NodePortId> {
        self.receivers().map(|(id, _)| id)
    }

    pub fn senders(&self) -> impl Iterator<Item = (&NodePortId, &NodePortSpec)> {
        self.iter()
            .filter(|(_, spec)| spec.kind == NodePortKind::Sender)
    }

    pub fn receivers(&self) -> impl Iterator<Item = (&NodePortId, &NodePortSpec)> {
        self.iter()
            .filter(|(_, spec)| spec.kind == NodePortKind::Receiver)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&NodePortId, &NodePortSpec)> {
        self.ids().iter().zip(self.specs())
    }

    pub fn spec(&self, id: NodePortId) -> Result<&NodePortSpec> {
        self.spec
            .get(id.raw_value() as usize)
            .ok_or_else(|| anyhow!("failed to obtain spec for port {id}"))
    }

    pub fn specs(&self) -> &[NodePortSpec] {
        &self.spec
    }
}

#[derive(Debug, Builder, Clone, Getters)]
pub struct NodeSpec {
    pub key: NodeSpecKey,
    //@todo value
    #[builder(default)]
    #[getset(get = "pub")]
    params: crate::spec::node::Schema,
    #[builder(default)]
    #[getset(get = "pub")]
    ports: PortsSpec,
}

impl NodeSpec {
    pub fn root() -> Self {
        NodeSpec::builder().key(NodeSpecKey::root()).build()
    }

    //@todo rework using getset macros
    pub fn key(&self) -> &NodeSpecKey {
        &self.key
    }

    pub fn name(&self) -> &NodeName {
        &self.key.name
    }

    pub fn kind(&self) -> NodeKind {
        self.key.kind
    }
}

#[derive(Debug, Default, Builder, Clone)]
pub struct NodeSpecValue {
    #[builder(default)]
    pub params: crate::spec::node::Schema,
    #[builder(default)]
    pub ports: PortsSpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters, CopyGetters, Serialize, Deserialize)]
pub struct NodeSpecKey {
    #[getset(get = "pub")]
    name: NodeName,
    #[getset(get_copy = "pub")]
    kind: NodeKind,
}

impl NodeSpecKey {
    pub fn new(name: NodeName, kind: NodeKind) -> Self {
        Self { name, kind }
    }

    pub fn root() -> Self {
        Self {
            name: NodeName::new("Root"),
            kind: NodeKind::Root,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelData {
    pub spec_id: ChannelSpecId,
    pub config: ChannelConfig,
}

impl ChannelData {
    pub fn new(spec_id: ChannelSpecId, config: ChannelConfig) -> Self {
        Self { spec_id, config }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NodeUiData {
    pub position: NodePosition,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ChannelUiData {
    pub position: ChannelPosition,
}

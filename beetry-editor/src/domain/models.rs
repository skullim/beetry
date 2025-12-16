use std::collections::HashSet;

use anyhow::{Result, anyhow, bail};
use beetry_core::MessageHash;
use beetry_plugin_types::{channel::MessageSpec, node::NodeName, parameter};
use beetry_reconstruction_types::channel::ChannelConfig;
use bon::Builder;
use serde::{Deserialize, Serialize};

pub type NodeId = usize;
pub type NodeSpecId = usize;
pub type NodePortId = u8;
pub type EdgeId = usize;
//@todo All Ids should be defined on the editor side, since that's the producing side
pub type ChannelId = beetry_reconstruction_types::channel::ChannelId;
pub type ChannelSpecId = usize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodePosition {
    origin: Point,
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

    pub fn connect(&mut self, id: ChannelId) -> Result<()> {
        match self {
            Self::Unconnected => *self = Self::Internal(<_>::from_iter(std::iter::once(id))),
            Self::Internal(connected) => {
                connected.insert(id);
            }
            Self::External => {
                bail!("attempted to connect {id} to external port")
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

pub struct ExternalReceivers {
    receivers: HashSet<MessageHash>,
}

pub struct ExternalSenders {
    senders: HashSet<MessageHash>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        let ids: Vec<_> = (0..spec.len()).map(|v| v as NodePortId).collect();
        Self { spec, ids }
    }

    pub fn ids(&self) -> &[NodePortId] {
        &self.ids
    }

    pub fn spec(&self, id: NodePortId) -> Result<&NodePortSpec> {
        self.spec
            .get(id as usize)
            .ok_or_else(|| anyhow!("failed to obtain spec for port {id}"))
    }

    pub fn specs(&self) -> &[NodePortSpec] {
        &self.spec
    }
}

#[derive(Debug, Builder, Clone, PartialEq, Eq, Hash)]
pub struct NodeSpec {
    pub name: NodeName,
    pub kind: NodeKind,
    #[builder(default)]
    pub param_schema: parameter::Schema,
    #[builder(default)]
    pub ports: PortsSpec,
}

pub struct ChannelDataInput {
    pub config: ChannelConfig,
    pub position: ChannelPosition,
}

pub struct ChannelData {
    pub spec_id: ChannelSpecId,
    pub config: ChannelConfig,
    pub position: ChannelPosition,
}

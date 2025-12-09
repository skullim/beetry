use std::collections::HashSet;

use beetry_core::MessageHash;
use beetry_serde::ser::{channel::MessageSpec, node::NodeName, parameter};
use bon::Builder;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub type NodeId = usize;
pub type NodeSpecId = usize;
pub type NodeChannelPortId = u8;
pub type EdgeId = usize;
pub type ChannelId = beetry_serde::de::channel::ChannelId;

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodePosition {
    origin: Point,
}

//@todo maybe use explicitly unconnected state initially and non empty hash set for internal connection kind
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodePortConnection {
    Internal(HashSet<ChannelId>), // connections
    External,
}

impl Default for NodePortConnection {
    fn default() -> Self {
        Self::Internal(Default::default())
    }
}

impl NodePortConnection {
    pub fn is_external(&self) -> bool {
        matches!(self, Self::External)
    }
}

pub enum NodePortKind {
    Sender,
    Receiver,
}

pub struct NodePortSpec {
    pub kind: NodePortKind,
    pub msg_spec: MessageSpec,
}

pub type ChannelPosition = NodePosition;

#[derive(Debug, Default, Clone, PartialEq, Props, Serialize, Deserialize)]
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
    senders: Vec<MessageSpec>,
    receivers: Vec<MessageSpec>,
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

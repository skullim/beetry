use std::collections::HashSet;

use beetry_core::MessageHash;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub type NodeId = usize;
pub type EdgeId = usize;
pub type ChannelId = usize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodePosition {
    origin: Point,
}

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

pub enum NodeKind {
    Action,
    Condition,
    Control,
    Decorator,
    Root,
}

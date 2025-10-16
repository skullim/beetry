use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub(crate) type NodeId = usize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct Point {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

#[derive(Debug, Default, Clone, PartialEq, Props, Serialize, Deserialize)]
pub(crate) struct PointEdge {
    pub(crate) start: Point,
    pub(crate) end: Point,
}

pub(crate) struct IndexedDragOffset {
    pub(crate) id: NodeId,
    pub(crate) offset: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct NodeEdge {
    pub(crate) from: NodeId,
    pub(crate) to: NodeId,
}

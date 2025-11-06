use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub type NodeId = usize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Default, Clone, PartialEq, Props, Serialize, Deserialize)]
pub struct PointEdge {
    pub start: Point,
    pub end: Point,
}

pub struct IndexedDragOffset {
    pub id: NodeId,
    pub offset: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeEdge {
    pub from: NodeId,
    pub to: NodeId,
}

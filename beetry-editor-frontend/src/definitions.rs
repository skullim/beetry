use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Default, Clone, PartialEq, Props, Serialize, Deserialize)]
pub struct EdgePos {
    pub start: Point,
    pub end: Point,
}

pub struct IndexedDragOffset {
    pub id: NodeId,
    pub offset: Point,
}

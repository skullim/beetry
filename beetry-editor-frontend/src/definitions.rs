use crate::Point;
use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Props, Serialize, Deserialize)]
pub struct EdgePos {
    pub start: Point,
    pub end: Point,
}

pub struct IndexedDragOffset {
    pub id: NodeId,
    pub offset: Point,
}

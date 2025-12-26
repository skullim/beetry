use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NodeUiData {
    pub position: NodePosition,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ChannelUiData {
    pub position: ChannelPosition,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodePosition {
    pub origin: Point,
}

pub type ChannelPosition = NodePosition;

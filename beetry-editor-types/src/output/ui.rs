use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NodeUiData {
    pub position: Point,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ChannelUiData {
    pub position: Point,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

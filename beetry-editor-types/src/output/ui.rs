use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NodeUiData {
    pub position: Point,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChannelUiData {
    pub position: Point,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PortConnectionUiData {
    visibility: VisibilityKind,
}

impl PortConnectionUiData {
    pub fn new(visibility: VisibilityKind) -> Self {
        Self { visibility }
    }

    pub fn visibility(&self) -> &VisibilityKind {
        &self.visibility
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum VisibilityKind {
    Visible,
    Hidden,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

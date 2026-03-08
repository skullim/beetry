use crate::Point;
use beetry_editor_types::id::{ChannelId, NodeId};
use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragNodeState {
    Idle,
    Dragged { id: NodeId, offset: Point },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragChannelState {
    Idle,
    Dragged { id: ChannelId, offset: Point },
}

#[derive(Clone, Copy, PartialEq)]
pub struct State {
    pub node: Signal<DragNodeState>,
    pub channel: Signal<DragChannelState>,
}

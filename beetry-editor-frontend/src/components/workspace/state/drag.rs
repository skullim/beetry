use crate::Point;
use beetry_editor_types::id::{ChannelId, NodeId};
use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DragNodeState {
    Idle,
    Dragged { id: NodeId, offset: Point },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DragChannelState {
    Idle,
    Dragged { id: ChannelId, offset: Point },
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct State {
    pub(crate) node: Signal<DragNodeState>,
    pub(crate) channel: Signal<DragChannelState>,
}

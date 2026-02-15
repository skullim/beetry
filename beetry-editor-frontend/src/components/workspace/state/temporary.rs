use crate::definitions::EdgePos;
use crate::ui::edge;
use crate::{Point, ui::node::ConnectionOrigin};
use beetry_editor_types::id::{NodeId, NodePortId};
use dioxus::prelude::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Dragged {
        from: NodeId,
        pos: EdgePos,
    },
}

impl From<&State> for edge::temporary::State {
    fn from(value: &State) -> Self {
        match value {
            State::Idle => Self::Idle,
            State::Dragged { pos, .. } => Self::Dragged { pos: *pos },
        }
    }
}

impl State {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_dragged(&mut self, from: NodeId, pos: EdgePos) {
        *self = Self::Dragged { from, pos };
    }

    pub(crate) fn take_dragged(&mut self) -> Option<NodeId> {
        let state = std::mem::take(self);
        if let Self::Dragged { from, .. } = state {
            return Some(from);
        }
        None
    }

    pub(crate) fn update_end_if_dragged(&mut self, evt: &Event<MouseData>) {
        if let Self::Dragged { from: _, pos } = self {
            let mouse_coords = evt.element_coordinates();
            pos.end = Point {
                x: mouse_coords.x,
                y: mouse_coords.y,
            };
        }
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum ChannelState {
    #[default]
    Idle,
    Dragged {
        data: DraggedData,
        pos: EdgePos,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraggedData {
    pub origin: ConnectionOrigin,
    pub node_id: NodeId,
    pub port_id: NodePortId,
}

impl From<&ChannelState> for edge::temporary::State {
    fn from(value: &ChannelState) -> Self {
        match value {
            ChannelState::Idle => Self::Idle,
            ChannelState::Dragged { pos, .. } => Self::Dragged { pos: *pos },
        }
    }
}

impl ChannelState {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_dragged(&mut self, data: DraggedData, pos: EdgePos) {
        *self = Self::Dragged { data, pos };
    }

    pub(crate) fn take_dragged(&mut self) -> Option<DraggedData> {
        let state = std::mem::take(self);
        if let Self::Dragged { data, .. } = state {
            return Some(data);
        }
        None
    }

    pub(crate) fn update_end_if_dragged(&mut self, evt: &Event<MouseData>) {
        if let Self::Dragged { data: _, pos } = self {
            let mouse_coords = evt.element_coordinates();
            pos.end = Point {
                x: mouse_coords.x,
                y: mouse_coords.y,
            };
        }
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

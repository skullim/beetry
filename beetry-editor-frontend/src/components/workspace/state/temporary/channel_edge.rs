use crate::definitions::EdgePos;
use crate::ui::edge;
use crate::{Point, ui::node::port::ConnectionOrigin};
use beetry_editor_types::id::{NodeId, NodePortId};

#[derive(Debug, Default, Clone, PartialEq)]
pub enum State {
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

    pub(crate) fn update_end_if_dragged(&mut self, cursor: Point) {
        if let Self::Dragged { pos, .. } = self {
            pos.end = cursor;
        }
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

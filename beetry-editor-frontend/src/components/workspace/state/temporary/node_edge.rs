use crate::definitions::EdgePos;
use crate::ui::edge;
use crate::Point;
use beetry_editor_types::id::NodeId;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Dragged { from: NodeId, pos: EdgePos },
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

    pub(crate) fn update_end_if_dragged(&mut self, cursor: Point) {
        if let Self::Dragged { from: _, pos } = self {
            pos.end = cursor;
        }
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

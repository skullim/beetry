use crate::Point;
use beetry_editor_types::{id::NodeId, id::NodePortId};
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::definitions::EdgePos;
use crate::ui::curve::Curve;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionOrigin {
    Sender,
    Receiver,
}

#[component]
pub fn Temporary(edge: ReadSignal<EdgePos>) -> Element {
    debug!("rendering temp channel connection with data: {edge:?}");
    let edge = edge.read();

    rsx! {
        path {
            d: "{Curve::calculate_horizontal(&edge.start, &edge.end)}",
            stroke: "#3a2020ff",
            stroke_width: "2",
            fill: "none",
            stroke_dasharray: "5,5",
            opacity: "0.7",
            style: "pointer-events: none",
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    #[default]
    Idle,
    Dragged(DraggedData),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraggedData {
    pub origin: ConnectionOrigin,
    pub node_id: NodeId,
    pub port_id: NodePortId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context {
    state: Signal<ConnectionState>,
    edge: Signal<EdgePos>,
}

impl Context {
    pub(crate) fn new() -> Self {
        Self {
            state: Signal::new(ConnectionState::Idle),
            edge: Signal::new(EdgePos::default()),
        }
    }

    pub(crate) fn is_dragged(&self) -> bool {
        matches!(*self.state.read(), ConnectionState::Dragged { .. })
    }

    pub(crate) fn set_dragged(&mut self, data: DraggedData) {
        self.state.set(ConnectionState::Dragged(data));
    }

    pub(crate) fn take_dragged(&mut self) -> Option<DraggedData> {
        if let ConnectionState::Dragged(data) = self.state.take() {
            return Some(data);
        }
        None
    }

    pub(crate) fn update_end_if_dragged(&mut self, evt: &Event<MouseData>) {
        if let ConnectionState::Dragged(..) = *self.state.peek() {
            let mouse_coords = evt.element_coordinates();
            self.edge.with_mut(|data| {
                data.end = Point {
                    x: mouse_coords.x,
                    y: mouse_coords.y,
                }
            });
        }
    }

    pub(crate) fn update_edge_pos(&mut self, mut pos: EdgePos) {
        self.edge.with_mut(|p| {
            std::mem::swap(p, &mut pos);
        })
    }

    pub(crate) fn edge(&self) -> EdgePos {
        self.edge.read().clone()
    }

    pub(crate) fn reset(&mut self) {
        self.state.set(ConnectionState::Idle);
    }
}

use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::{
    definitions::{NodeId, Point, PointEdge},
    ui::curve::Curve,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context {
    state: Signal<ConnectionState>,
    edge: Signal<PointEdge>,
}

impl Context {
    pub(crate) fn new() -> Self {
        Self {
            state: Signal::new(ConnectionState::Idle),
            edge: Signal::new(PointEdge::default()),
        }
    }

    pub(crate) fn is_dragged(&self) -> bool {
        matches!(*self.state.read(), ConnectionState::Dragged { from: _ })
    }

    pub(crate) fn set_dragged_from(&mut self, from: NodeId) {
        self.state.set(ConnectionState::Dragged { from });
    }

    pub(crate) fn take_dragged(&mut self) -> Option<NodeId> {
        if let ConnectionState::Dragged { from } = self.state.take() {
            return Some(from);
        }
        None
    }

    pub(crate) fn update_end_if_dragged(&mut self, evt: &Event<MouseData>) {
        if let ConnectionState::Dragged { from: _ } = *self.state.peek() {
            let mouse_coords = evt.element_coordinates();
            self.edge.with_mut(|data| {
                data.end = Point {
                    x: mouse_coords.x,
                    y: mouse_coords.y,
                }
            });
        }
    }

    pub(crate) fn update_edge(&mut self, mut pos: PointEdge) {
        self.edge.with_mut(|p| {
            std::mem::swap(p, &mut pos);
        })
    }

    pub(crate) fn edge(&self) -> PointEdge {
        self.edge.read().clone()
    }

    pub(crate) fn reset(&mut self) {
        self.state.set(ConnectionState::Idle);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Idle,
    Dragged { from: NodeId },
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Idle
    }
}

#[component]
pub fn Temporary(edge: ReadSignal<PointEdge>) -> Element {
    debug!("rendering temp edge component with data: {edge:?}");
    let edge = edge.read();

    rsx! {
        path {
            d: "{Curve::calculate_vertical(&edge.start, &edge.end)}",
            stroke: "#A78BFA", // Light purple to match output port hover
            stroke_width: "2",
            fill: "none",
            stroke_dasharray: "5,5",
            opacity: "0.7",
            style: "pointer-events: none",
        }
    }
}

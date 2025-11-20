use beetry_core::MessageHash;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::definitions::{EdgePos, NodeId, Point};
use crate::ui::curve::Curve;

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
        matches!(
            *self.state.read(),
            ConnectionState::Dragged {
                from: _,
                msg_hash: _,
                origin: _,
            }
        )
    }

    pub(crate) fn set_dragged(
        &mut self,
        origin: ConnectionOrigin,
        from: NodeId,
        msg_hash: MessageHash,
    ) {
        self.state.set(ConnectionState::Dragged {
            origin,
            from,
            msg_hash,
        });
    }

    pub(crate) fn take_dragged(&mut self) -> Option<(ConnectionOrigin, NodeId, MessageHash)> {
        if let ConnectionState::Dragged {
            origin,
            from,
            msg_hash,
        } = self.state.take()
        {
            return Some((origin, from, msg_hash));
        }
        None
    }

    pub(crate) fn update_end_if_dragged(&mut self, evt: &Event<MouseData>) {
        if let ConnectionState::Dragged {
            origin: _,
            from: _,
            msg_hash: _,
        } = *self.state.peek()
        {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionOrigin {
    Sender,
    Receiver,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    #[default]
    Idle,
    Dragged {
        origin: ConnectionOrigin,
        from: NodeId,
        msg_hash: MessageHash,
    },
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

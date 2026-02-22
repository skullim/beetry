use crate::{Point, ui::handler::define_handlers};
use beetry_editor_types::id::{ChannelId, NodeId, NodePortId};
use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionId {
    pub node_id: NodeId,
    pub port_id: NodePortId,
    pub channel_id: ChannelId,
}

define_handlers!(
    on_delete: ConnectionId,
    on_close: (),
);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Visible {
        position: Point,
        connection: ConnectionId,
    },
}

#[component]
pub fn Menu(state: ReadSignal<State>) -> Element {
    const WIDTH: u16 = 160;
    const HEIGHT: u16 = 36;

    let (position, connection) = match *state.read() {
        State::Idle => return rsx!(),
        State::Visible {
            position,
            connection,
        } => (position, connection),
    };

    let handlers = use_context::<Handlers>();
    rsx! {
        g { transform: "translate({position.x} {position.y})",
            rect {
                x: "0",
                y: "0",
                width: "{WIDTH}",
                height: "{HEIGHT}",
                fill: "white",
                stroke: "#ccc",
                style: "cursor: pointer;",
                onclick: move |evt| {
                    evt.stop_propagation();
                    handlers.on_delete.call(connection);
                    handlers.on_close.call(());
                },
                onmouseup: move |evt| {
                    evt.stop_propagation();
                },
            }

            text {
                x: "12",
                y: "24",
                fill: "#111",
                style: "pointer-events: none;",
                "Delete Edge"
            }
        }
    }
}

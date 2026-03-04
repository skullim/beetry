use crate::{Point, ui::handler::define_handlers};
use beetry_editor_types::id::ChannelEdgeId;
use dioxus::prelude::*;

define_handlers!(
    on_delete: ChannelEdgeId,
    on_close: (),
);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Visible {
        position: Point,
        edge: ChannelEdgeId,
    },
}

#[component]
pub fn Menu(state: ReadSignal<State>) -> Element {
    const WIDTH: u16 = 160;
    const HEIGHT: u16 = 36;

    let (position, edge) = match *state.read() {
        State::Idle => return rsx!(),
        State::Visible {
            position,
            edge,
        } => (position, edge),
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
                    handlers.on_delete.call(edge);
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

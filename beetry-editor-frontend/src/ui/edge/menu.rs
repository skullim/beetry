use beetry_editor_types::id::EdgeId;
use dioxus::prelude::*;

use crate::{Point, ui::handler::define_handlers};

define_handlers!(on_delete: EdgeId,
          on_close: (),
);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Visible {
        position: Point,
        edge_id: EdgeId,
    },
}

#[component]
pub fn Menu(state: ReadSignal<State>) -> Element {
    const WIDTH: u16 = 160;
    const HEIGHT: u16 = 36;
    debug!("rendering");
    let (position, edge_id) = match *state.read() {
        State::Idle => return rsx!(),
        State::Visible { position, edge_id } => (position, edge_id),
    };

    let menu_handlers = use_context::<Handlers>();
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
                    menu_handlers.on_delete.call(edge_id);
                    menu_handlers.on_close.call(());
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

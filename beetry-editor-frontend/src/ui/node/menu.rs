use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;

use crate::{Point, ui::handler::define_handlers};

define_handlers!(on_delete: NodeId,
          on_update: NodeId,
          on_close: (),
);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Visible {
        position: Point,
        id: NodeId,
        can_edit_params: bool,
    },
}

#[component]
pub fn Menu(state: ReadSignal<State>) -> Element {
    const WIDTH: u16 = 160;
    const ROW_HEIGHT: u16 = 36;

    debug!("rendering");
    let state_read = state.read();
    let (position, id, can_edit_params) = match *state_read {
        State::Idle => return rsx!(),
        State::Visible {
            position,
            id,
            can_edit_params,
        } => (position, id, can_edit_params),
    };

    let menu_handlers = use_context::<Handlers>();

    let menu_height = if can_edit_params {
        ROW_HEIGHT * 2
    } else {
        ROW_HEIGHT
    };

    rsx! {
        g { transform: "translate({position.x} {position.y})",
            rect {
                x: "0",
                y: "0",
                width: "{WIDTH}",
                height: "{menu_height}",
                fill: "var(--bt-menu-fill)",
                stroke: "var(--bt-menu-stroke)",
                style: "cursor: pointer;",
                onclick: move |evt| {
                    evt.stop_propagation();
                    menu_handlers.on_delete.call(id);
                    menu_handlers.on_close.call(());
                },
                onmouseup: move |evt| {
                    evt.stop_propagation();
                },
            }

            text {
                x: "12",
                y: "24",
                fill: "var(--bt-menu-text)",
                style: "pointer-events: none;",
                "Delete Node"
            }

            if can_edit_params {
                rect {
                    x: "0",
                    y: "{ROW_HEIGHT}",
                    width: "{WIDTH}",
                    height: "{ROW_HEIGHT}",
                    fill: "var(--bt-menu-fill)",
                    stroke: "var(--bt-menu-stroke)",
                    style: "cursor: pointer;",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        menu_handlers.on_update.call(id);
                        menu_handlers.on_close.call(());
                    },
                    onmouseup: move |evt| {
                        evt.stop_propagation();
                    },
                }

                text {
                    x: "12",
                    y: "60",
                    fill: "var(--bt-menu-text)",
                    style: "pointer-events: none;",
                    "Update Parameters"
                }
            }
        }
    }
}

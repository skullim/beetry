use crate::Point;
use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;
#[derive(Debug, Clone)]
pub struct Handlers {
    on_delete: EventHandler<NodeId>,
    on_edit_params: EventHandler<NodeId>,
    on_close: EventHandler<()>,
}

impl Handlers {
    pub fn new(
        on_delete: impl FnMut(NodeId) -> Result<()> + 'static,
        on_edit_params: impl FnMut(NodeId) + 'static,
        on_close: impl FnMut(()) + 'static,
    ) -> Self {
        Self {
            on_delete: EventHandler::new(on_delete),
            on_edit_params: EventHandler::new(on_edit_params),
            on_close: EventHandler::new(on_close),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Visible {
        position: Point,
        node_id: NodeId,
        can_edit_params: bool,
    },
}

#[derive(Debug, Props, PartialEq, Clone)]
pub struct ContextMenuProps {
    state: ReadSignal<State>,
}

#[component]
pub fn ContextMenu(props: ContextMenuProps) -> Element {
    debug!("rendering");
    let state_read = props.state.read();
    let (position, node_id, can_edit_params) = match *state_read {
        State::Idle => return rsx!(),
        State::Visible {
            position,
            node_id,
            can_edit_params,
        } => (position, node_id, can_edit_params),
    };

    let context_menu_handlers = use_context::<Handlers>();

    let menu_width = 160;
    let menu_height = if can_edit_params { 72 } else { 36 };

    rsx! {
        g { transform: "translate({position.x} {position.y})",
            rect {
                x: "0",
                y: "0",
                width: "{menu_width}",
                height: "{menu_height}",
                fill: "white",
                stroke: "#ccc",
                style: "cursor: pointer;",
                onclick: move |evt| {
                    evt.stop_propagation();
                    context_menu_handlers.on_delete.call(node_id);
                    context_menu_handlers.on_close.call(());
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
                "Delete Node"
            }

            if can_edit_params {
                rect {
                    x: "0",
                    y: "36",
                    width: "{menu_width}",
                    height: "36",
                    fill: "white",
                    stroke: "#ccc",
                    style: "cursor: pointer;",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        context_menu_handlers.on_edit_params.call(node_id);
                        context_menu_handlers.on_close.call(());
                    },
                    onmouseup: move |evt| {
                        evt.stop_propagation();
                    },
                }

                text {
                    x: "12",
                    y: "60",
                    fill: "#111",
                    style: "pointer-events: none;",
                    "Edit Parameters"
                }
            }
        }
    }
}

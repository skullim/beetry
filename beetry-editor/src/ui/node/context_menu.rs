use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::definitions::{NodeId, Point};

#[derive(Debug, Clone)]
pub(crate) struct Handlers {
    on_delete: EventHandler<NodeId>,
    on_close: EventHandler<()>,
}

impl Handlers {
    pub(crate) fn new(
        on_delete: impl FnMut(NodeId) + 'static,
        on_close: impl FnMut(()) + 'static,
    ) -> Self {
        Self {
            on_delete: EventHandler::new(on_delete),
            on_close: EventHandler::new(on_close),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(crate) struct State {
    pub(crate) position: Point,
    pub(crate) target_node: NodeId,
    pub(crate) is_visible: bool,
}

#[derive(Debug, Props, PartialEq, Clone)]
pub(crate) struct ContextMenuProps {
    state: ReadSignal<State>,
}

#[component]
pub(crate) fn ContextMenu(props: ContextMenuProps) -> Element {
    debug!("rendering context menu");
    let state_read = props.state.read();

    let position = &state_read.position;
    let target_node = state_read.target_node;
    let is_visible = state_read.is_visible;

    let context_menu_handlers = use_context::<Handlers>();

    rsx! {
        if is_visible {
            div {
                position: "fixed",
                top: "0",
                left: "0",
                width: "100vw",
                height: "100vh",
                z_index: "1000",
                onclick: move |_| context_menu_handlers.on_close.call(()),

                div {
                    position: "absolute",
                    left: "{position.x}px",
                    top: "{position.y}px",
                    background: "white",
                    border: "1px solid #ccc",
                    border_radius: "4px",
                    box_shadow: "0 2px 8px rgba(0,0,0,0.2)",
                    min_width: "120px",
                    z_index: "1001",
                    onclick: move |evt| evt.stop_propagation(),

                    div {
                        padding: "8px 16px",
                        cursor: "pointer",
                        onclick: move |_| {
                            context_menu_handlers.on_delete.call(target_node);
                            context_menu_handlers.on_close.call(());
                        },
                        "Delete Node"
                    }
                }
            }
        }
    }
}

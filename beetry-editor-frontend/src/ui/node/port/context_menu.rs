use beetry_editor_types::{
    id::{NodeId, NodePortId},
    output::ui::Point,
};
use bon::bon;
use dioxus::prelude::*;

#[derive(Debug, Clone)]
pub(crate) struct Handlers {
    on_internal: EventHandler<(NodeId, NodePortId)>,
    on_external: EventHandler<(NodeId, NodePortId)>,
}

#[bon]
impl Handlers {
    #[builder]
    pub(crate) fn new(
        on_internal: impl FnMut((NodeId, NodePortId)) + 'static,
        on_external: impl FnMut((NodeId, NodePortId)) + 'static,
    ) -> Self {
        Self {
            on_internal: EventHandler::new(on_internal),
            on_external: EventHandler::new(on_external),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) enum State {
    #[default]
    Idle,
    Visible {
        position: Point,
        id: NodeId,
        port_id: NodePortId,
    },
}

#[component]
pub fn PortContextMenu(state: Signal<State>) -> Element {
    debug!("rendering");
    let mut checked = use_signal(|| false);

    match state() {
        State::Idle => {
            rsx! {}
        }
        State::Visible {
            position,
            id,
            port_id,
        } => {
            rsx! {
                g {
                    transform: "translate({position.x} {position.y})",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        checked.toggle();
                        if *checked.peek() {
                            use_context::<Handlers>().on_external.call((id, port_id));
                        } else {
                            use_context::<Handlers>().on_internal.call((id, port_id));
                        }
                    },
                    onmouseup: move |evt| {
                        evt.stop_propagation();
                    },

                    rect {
                        x: "-8",
                        y: "-8",
                        width: "160",
                        height: "32",
                        rx: "6",
                        ry: "6",
                        fill: "white",
                        stroke: "#ddd",
                        pointer_events: "all",
                    }

                    // box to select port type
                    rect {
                        x: "0",
                        y: "0",
                        width: "16",
                        height: "16",
                        rx: "3",
                        ry: "3",
                        fill: "white",
                        stroke: "#444",
                        stroke_width: "1",
                        style: "cursor: pointer;",
                    }

                    if checked() {
                        path {
                            d: "M3 8 L7 12 L13 4",
                            fill: "none",
                            stroke: "#1a73e8",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                        }
                    }

                    text {
                        x: "24",
                        y: "12",
                        font_size: "12",
                        fill: "#222",
                        dominant_baseline: "middle",
                        "External port"
                    }
                }
            }
        }
    }
}

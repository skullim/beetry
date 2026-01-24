use beetry_editor_types::id::{NodeId, NodePortId};
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
pub(super) enum State {
    #[default]
    Idle,
    Visible {
        id: NodeId,
        port_id: NodePortId,
    },
}

#[component]
pub(super) fn PortSettingsPopup(state: Signal<State>, is_external: Signal<bool>) -> Element {
    debug!("rendering");
    debug!("{state:?}");
    match state() {
        State::Idle => {
            rsx! {}
        }
        State::Visible { id, port_id } => {
            rsx! {
                div {
                    position: "fixed",
                    top: "0",
                    left: "0",
                    width: "100vw",
                    height: "100vh",
                    z_index: "1000",

                    div {
                        position: "absolute",
                        background: "white",
                        border: "1px solid #ccc",
                        border_radius: "8px",
                        box_shadow: "0 4px 16px rgba(0,0,0,0.3)",
                        min_width: "320px",
                        max_width: "500px",
                        padding: "20px",
                        z_index: "1001",
                        onclick: move |evt| evt.stop_propagation(),

                        h3 { margin: "0 0 16px 0", "port settings" }


                        label {
                            input {
                                r#type: "checkbox",
                                checked: "{is_external()}",
                                onchange: move |evt| {
                                    if evt.checked() {
                                        is_external.set(true);
                                        use_context::<Handlers>().on_internal.call((id, port_id));
                                    } else {
                                        is_external.set(false);
                                        use_context::<Handlers>().on_external.call((id, port_id));
                                    }
                                },
                            }
                            "External port"
                        }

                        div {
                            padding: "8px 16px",
                            border: "1px solid #ddd",
                            border_radius: "4px",
                            background: "white",
                            cursor: "pointer",
                            button {
                                onclick: move |_| {
                                    state.set(State::Idle);
                                },
                                "Close"
                            }
                        }
                    }
                }
            }
        }
    }
}

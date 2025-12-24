use beetry_plugin_types::channel::ChannelSpec;
use beetry_reconstruction_types::channel::{ChannelConfig, ChannelKind, TokioChannelKind};
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::definitions::Point;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Visible {
        position: Point,
        spec: ChannelSpec,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ChannelType {
    Mpsc,
    Broadcast,
}

#[derive(Debug, Props, Clone, PartialEq, Eq)]
pub struct DialogProps {
    state: Signal<State>,
}

#[component]
pub fn Dialog2(props: DialogProps) -> Element {
    debug!("rendering channel config dialog");
    let state_read = props.state.read();

    let position = match *state_read {
        State::Idle => return rsx! {},
        State::Visible { position, spec: _ } => position,
    };

    let mut capacity = use_signal(|| 1usize);
    let mut channel_type = use_signal(|| ChannelType::Mpsc);

    let handlers = use_context::<Handlers2>();

    let on_confirm = move |_| {
        let capacity_val = *capacity.read();
        let type_val = *channel_type.read();
        let impl_kind = match type_val {
            ChannelType::Mpsc => ChannelKind::Tokio(TokioChannelKind::Mpsc),
            ChannelType::Broadcast => ChannelKind::Tokio(TokioChannelKind::Broadcast),
        };

        let channel_metadata = ChannelConfig::new(capacity_val, impl_kind);
        handlers.on_confirm.call(channel_metadata);
    };

    let on_cancel = move |_| {
        handlers.on_cancel.call(());
    };

    rsx! {
        div {
            position: "fixed",
            top: "0",
            left: "0",
            width: "100vw",
            height: "100vh",
            background: "rgba(0,0,0,0.5)",
            z_index: "1000",
            onclick: on_cancel,

            div {
                position: "absolute",
                left: "{position.x}px",
                top: "{position.y}px",
                background: "white",
                border: "1px solid #ccc",
                border_radius: "8px",
                box_shadow: "0 4px 16px rgba(0,0,0,0.3)",
                min_width: "320px",
                padding: "20px",
                z_index: "1001",
                onclick: move |evt| evt.stop_propagation(),

                h3 { margin: "0 0 16px 0", "Channel Configuration" }

                div { margin_bottom: "12px",
                    label {
                        display: "block",
                        margin_bottom: "4px",
                        font_weight: "bold",
                        "Capacity:"
                    }
                    input {
                        r#type: "number",
                        min: "0",
                        value: "{capacity}",
                        width: "100%",
                        padding: "4px 8px",
                        border: "1px solid #ddd",
                        border_radius: "4px",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<usize>() && val > 0 {
                                capacity.set(val);
                            }
                        },
                    }
                }

                div { margin_bottom: "12px",
                    label {
                        display: "block",
                        margin_bottom: "4px",
                        font_weight: "bold",
                        "Channel Type:"
                    }
                    select {
                        width: "100%",
                        padding: "4px 8px",
                        border: "1px solid #ddd",
                        border_radius: "4px",
                        onchange: move |evt| {
                            match evt.value().as_str() {
                                "Mpsc" => channel_type.set(ChannelType::Mpsc),
                                "Broadcast" => channel_type.set(ChannelType::Broadcast),
                                _ => {}
                            }
                        },
                        option {
                            value: "Mpsc",
                            selected: matches!(*channel_type.read(), ChannelType::Mpsc),
                            "Multi-Producer, Single-Consumer (MPSC)"
                        }
                        option {
                            value: "Broadcast",
                            selected: matches!(*channel_type.read(), ChannelType::Broadcast),
                            "Broadcast"
                        }
                    }
                }

                div {
                    display: "flex",
                    justify_content: "flex-end",
                    gap: "8px",
                    margin_top: "20px",

                    button {
                        padding: "8px 16px",
                        border: "1px solid #ddd",
                        border_radius: "4px",
                        background: "white",
                        cursor: "pointer",
                        onclick: on_cancel,
                        "Cancel"
                    }

                    button {
                        padding: "8px 16px",
                        border: "1px solid #007acc",
                        border_radius: "4px",
                        background: "#007acc",
                        color: "white",
                        cursor: "pointer",
                        onclick: on_confirm,
                        "Confirm"
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Handlers2 {
    pub(crate) on_confirm: EventHandler<ChannelConfig>,
    pub(crate) on_cancel: EventHandler<()>,
}

impl Handlers2 {
    pub(crate) fn new(
        on_confirm: impl FnMut(ChannelConfig) + 'static,
        on_cancel: impl FnMut(()) + 'static,
    ) -> Self {
        Self {
            on_confirm: EventHandler::new(on_confirm),
            on_cancel: EventHandler::new(on_cancel),
        }
    }
}

use beetry_serde::de::channel::{
    BroadcastConfig, ChannelImplKind, ChannelKind, ChannelMetadata, MpscConfig, TokioChannelConfig,
};
use beetry_serde::ser::channel::ChannelSpec;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;
use std::num::NonZeroUsize;

use crate::definitions::Point;

#[derive(Debug, Clone)]
pub struct Handlers {
    pub(crate) on_confirm: EventHandler<ChannelMetadata>,
    pub(crate) on_cancel: EventHandler<()>,
}

impl Handlers {
    pub(crate) fn new(
        on_confirm: impl FnMut(ChannelMetadata) + 'static,
        on_cancel: impl FnMut(()) + 'static,
    ) -> Self {
        Self {
            on_confirm: EventHandler::new(on_confirm),
            on_cancel: EventHandler::new(on_cancel),
        }
    }
}

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
pub fn Dialog(props: DialogProps) -> Element {
    debug!("rendering channel config dialog");
    let state_read = props.state.read();

    let position = match *state_read {
        State::Idle => return rsx! {},
        State::Visible { position, spec: _ } => position,
    };

    let mut capacity = use_signal(|| 1usize);
    let mut channel_kind = use_signal(|| ChannelKind::Internal);
    let mut channel_type = use_signal(|| ChannelType::Mpsc);
    let mut n_senders = use_signal(|| 1usize);
    let mut n_receivers = use_signal(|| 1usize);

    let handlers = use_context::<Handlers>();

    let on_confirm = move |_| {
        let capacity_val = *capacity.read();
        let kind_val = *channel_kind.read();
        let type_val = *channel_type.read();
        let senders_val = *n_senders.read();
        let receivers_val = *n_receivers.read();

        let non_zero = NonZeroUsize::new(1).unwrap();
        let impl_kind = match type_val {
            ChannelType::Mpsc => ChannelImplKind::Tokio(TokioChannelConfig::Mpsc(MpscConfig::new(
                NonZeroUsize::new(senders_val).unwrap_or(non_zero),
            ))),
            ChannelType::Broadcast => ChannelImplKind::Tokio(TokioChannelConfig::Broadcast(
                BroadcastConfig::builder()
                    .n_senders(NonZeroUsize::new(senders_val).unwrap_or(non_zero))
                    .n_receivers(NonZeroUsize::new(receivers_val).unwrap_or(non_zero))
                    .build(),
            )),
        };

        let channel_metadata = ChannelMetadata::new(capacity_val, kind_val, impl_kind);
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
                            if let Ok(val) = evt.value().parse::<usize>() &&
                                val > 0 {
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
                        "Channel Kind:"
                    }
                    select {
                        width: "100%",
                        padding: "4px 8px",
                        border: "1px solid #ddd",
                        border_radius: "4px",
                        onchange: move |evt| {
                            match evt.value().as_str() {
                                "Internal" => channel_kind.set(ChannelKind::Internal),
                                "External" => channel_kind.set(ChannelKind::External),
                                _ => {}
                            }
                        },
                        option {
                            value: "Internal",
                            selected: matches!(*channel_kind.read(), ChannelKind::Internal),
                            "Internal"
                        }
                        option {
                            value: "External",
                            selected: matches!(*channel_kind.read(), ChannelKind::External),
                            "External"
                        }
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

                div { margin_bottom: "12px",
                    label {
                        display: "block",
                        margin_bottom: "4px",
                        font_weight: "bold",
                        "Number of Senders:"
                    }
                    input {
                        r#type: "number",
                        min: "1",
                        value: "{n_senders}",
                        width: "100%",
                        padding: "4px 8px",
                        border: "1px solid #ddd",
                        border_radius: "4px",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<usize>() &&
                                val > 0 {
                                    n_senders.set(val);
                                }
                        },
                    }
                }

                if matches!(*channel_type.read(), ChannelType::Broadcast) {
                    div { margin_bottom: "12px",
                        label {
                            display: "block",
                            margin_bottom: "4px",
                            font_weight: "bold",
                            "Number of Receivers:"
                        }
                        input {
                            r#type: "number",
                            min: "1",
                            value: "{n_receivers}",
                            width: "100%",
                            padding: "4px 8px",
                            border: "1px solid #ddd",
                            border_radius: "4px",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<usize>() &&
                                    val > 0 {
                                        n_receivers.set(val);
                                    }
                            },
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

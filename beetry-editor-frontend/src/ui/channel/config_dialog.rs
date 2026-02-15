use crate::{Point, SharedSpecs, ui::handler::handlers};
use beetry_core::MessageHash;
use beetry_editor_types::output::channel::{ChannelConfig, ChannelKind, TokioChannelKind};
use dioxus::prelude::*;
use dioxus_logger::tracing::{debug, error};

handlers!(on_confirm: (MessageHash, ChannelConfig),
          on_cancel: (),
);

#[derive(Debug, Default, Clone, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Visible {
        position: Point,
        spec_key: MessageHash,
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
    debug!("rendering");
    let state_read = props.state.read();

    let (position, spec_key) = match *state_read {
        State::Idle => return rsx! {},
        State::Visible { position, spec_key } => (position, spec_key),
    };

    let specs = use_context::<SharedSpecs>();
    if let Err(err) = specs.channels.spec(&spec_key) {
        error!("failed to open channel config dialog: {err:?}");
        return rsx! {};
    };

    let mut capacity = use_signal(|| 1usize);
    let mut channel_type = use_signal(|| ChannelType::Mpsc);

    let handlers = use_context::<Handlers>();

    let on_confirm = move |_| {
        let capacity = *capacity.read();
        let ty = *channel_type.read();
        let channel_kind = match ty {
            ChannelType::Mpsc => ChannelKind::Tokio(TokioChannelKind::Mpsc),
            ChannelType::Broadcast => ChannelKind::Tokio(TokioChannelKind::Broadcast),
        };

        let channel_config = ChannelConfig::new(capacity, channel_kind);
        handlers.on_confirm.call((spec_key, channel_config));
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
                            if let Ok(val) = evt.value().parse::<usize>() {
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

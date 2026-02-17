use crate::{Point, SharedSpecs, ui::handler::define_handlers};
use beetry_core::MessageHash;
use beetry_editor_types::output::channel::{ChannelConfig, ChannelKind, TokioChannelKind};
use dioxus::prelude::*;
use dioxus_logger::tracing::{debug, error};

define_handlers!(on_confirm: (MessageHash, ChannelConfig),
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
        div { class: "bt-dialog-overlay", onclick: on_cancel,

            div {
                class: "bt-dialog bt-dialog--channel",
                left: "{position.x}px",
                top: "{position.y}px",
                onclick: move |evt| evt.stop_propagation(),

                h3 { class: "bt-dialog-title", "Channel Configuration" }

                div { class: "bt-form-field",
                    label { class: "bt-form-label", "Capacity:" }
                    input {
                        class: "bt-form-input",
                        r#type: "number",
                        min: "0",
                        value: "{capacity}",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<usize>() {
                                capacity.set(val);
                            }
                        },
                    }
                }

                div { class: "bt-form-field",
                    label { class: "bt-form-label", "Channel Type:" }
                    select {
                        class: "bt-form-input",
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

                div { class: "bt-dialog-actions",

                    button {
                        class: "bt-btn bt-btn--dialog-secondary",
                        onclick: on_cancel,
                        "Cancel"
                    }

                    button {
                        class: "bt-btn bt-btn--dialog-primary",
                        onclick: on_confirm,
                        "Confirm"
                    }
                }
            }
        }
    }
}

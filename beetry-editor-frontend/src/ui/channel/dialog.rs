use crate::{Point, ui::handler::define_handlers};
use beetry_core::MessageHash;
use beetry_editor_types::id::ChannelId;
use beetry_editor_types::output::channel::{
    ChannelConfig, ChannelConfigInput, ChannelConfigUpdate, ChannelKind, TokioChannelKind,
};
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

pub const DEFAULT_POSITION: Point = Point { x: 200.0, y: 100.0 };

define_handlers!(on_confirm: ConfirmAction,
                 on_cancel: (),
);

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    Create {
        spec_key: MessageHash,
        input: ChannelConfigInput,
    },
    Update {
        channel_id: ChannelId,
        update: ChannelConfigUpdate,
    },
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Visible {
        position: Point,
        mode: Mode,
        // Current config snapshot at Dialog render time
        // Dialog is free to mutate the config, but to write back
        // to backend it has to convert to concrete DTO expected by backend API.
        // This allows to precisely control what config fields are updatable.
        config: CopyValue<ChannelConfig>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Create { spec_key: MessageHash },
    Update { channel_id: ChannelId },
}

#[component]
pub fn Dialog(state: Signal<State>) -> Element {
    debug!("rendering");
    let state_read = state.read();

    let (position, mode, mut config) = match *state_read {
        State::Idle => return rsx! {},
        State::Visible {
            position,
            mode,
            config,
        } => (position, mode, config),
    };

    let handlers = use_context::<Handlers>();

    let on_confirm = move |_| {
        let config = config.peek().cloned();
        match mode {
            Mode::Create { spec_key } => {
                handlers.on_confirm.call(ConfirmAction::Create {
                    spec_key,
                    input: config.into(),
                });
            }
            Mode::Update { channel_id } => handlers.on_confirm.call(ConfirmAction::Update {
                channel_id,
                update: config.into(),
            }),
        }
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
                        value: "{config.peek().capacity()}",
                        min: "0",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<usize>() {
                                config.with_mut(|c| {c.set_capacity(val);})
                            }
                        },
                    }
                }

                if matches!(mode, Mode::Create { .. }) {
                    div { class: "bt-form-field",
                        label { class: "bt-form-label", "Channel Type:" }
                        select {
                            class: "bt-form-input",
                            onchange: move |evt| {
                                match evt.value().as_str() {
                                    "Mpsc" =>  config.with_mut(|c| {c.set_kind(ChannelKind::Tokio(TokioChannelKind::Mpsc));}),
                                    "Broadcast" => config.with_mut(|c| {c.set_kind(ChannelKind::Tokio(TokioChannelKind::Broadcast));}),
                                    "Watch" => config.with_mut(|c| {c.set_kind(ChannelKind::Tokio(TokioChannelKind::Watch));}),
                                    _ => {}
                                }
                            },
                            option {
                                value: "Mpsc",
                                selected: matches!(config.peek().kind(), ChannelKind::Tokio(TokioChannelKind::Mpsc)),
                                "Multi-Producer, Single-Consumer (MPSC)"
                            }
                            option {
                                value: "Broadcast",
                                selected: matches!(config.peek().kind(), ChannelKind::Tokio(TokioChannelKind::Broadcast)),
                                "Broadcast"
                            }
                            option {
                                value: "Watch",
                                selected: matches!(config.peek().kind(), ChannelKind::Tokio(TokioChannelKind::Watch)),
                                "Watch"
                            }
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

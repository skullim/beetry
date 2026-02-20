use crate::{Point, ui::handler::define_handlers};
use beetry_editor_types::id::ChannelId;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

define_handlers!(on_delete: ChannelId,
          on_close: (),
);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Visible {
        position: Point,
        channel_id: ChannelId,
    },
}

#[component]
pub fn Menu(state: ReadSignal<State>) -> Element {
    const WIDTH: u16 = 160;
    const HEIGHT: u16 = 36;
    debug!("rendering");
    let state_read = state.read();
    let (position, channel_id) = match *state_read {
        State::Idle => return rsx!(),
        State::Visible {
            position,
            channel_id,
        } => (position, channel_id),
    };

    let handlers = use_context::<Handlers>();

    rsx! {
        g { transform: "translate({position.x} {position.y})",
            rect {
                x: "0",
                y: "0",
                width: "{WIDTH}",
                height: "{HEIGHT}",
                fill: "white",
                stroke: "#ccc",
                style: "cursor: pointer;",
                onclick: move |evt| {
                    evt.stop_propagation();
                    handlers.on_delete.call(channel_id);
                    handlers.on_close.call(());
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
                "Delete Channel"
            }
        }
    }
}

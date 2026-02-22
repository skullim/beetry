use crate::{Point, ui::handler::define_handlers};
use beetry_editor_types::id::ChannelId;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

define_handlers!(on_delete: ChannelId,
                 on_update: ChannelId,
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
    const ROW_HEIGHT: u16 = 36;
    const SECOND_ROW_Y: u16 = ROW_HEIGHT;
    const TEXT_X: u16 = 12;
    const FIRST_TEXT_Y: u16 = 24;
    const SECOND_TEXT_Y: u16 = FIRST_TEXT_Y + SECOND_ROW_Y;
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
                height: "{ROW_HEIGHT}",
                fill: "var(--bt-menu-fill)",
                stroke: "var(--bt-menu-stroke)",
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
                x: "{TEXT_X}",
                y: "{FIRST_TEXT_Y}",
                fill: "var(--bt-menu-text)",
                style: "pointer-events: none;",
                "Delete Channel"
            }

            rect {
                x: "0",
                y: "{SECOND_ROW_Y}",
                width: "{WIDTH}",
                height: "{ROW_HEIGHT}",
                fill: "var(--bt-menu-fill)",
                stroke: "var(--bt-menu-stroke)",
                style: "cursor: pointer;",
                onclick: move |evt| {
                    evt.stop_propagation();
                    handlers.on_update.call(channel_id);
                    handlers.on_close.call(());
                },
                onmouseup: move |evt| {
                    evt.stop_propagation();
                },
            }

            text {
                x: "{TEXT_X}",
                y: "{SECOND_TEXT_Y}",
                fill: "var(--bt-menu-text)",
                style: "pointer-events: none;",
                "Update Configuration"
            }
        }
    }
}

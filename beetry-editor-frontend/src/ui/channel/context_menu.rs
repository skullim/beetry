use crate::{Point, ui::handler::handlers};
use beetry_editor_types::id::ChannelId;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

handlers!(on_delete: ChannelId,
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

#[derive(Debug, Props, PartialEq, Clone)]
pub struct ContextMenuProps {
    state: ReadSignal<State>,
}

#[component]
pub fn ContextMenu(props: ContextMenuProps) -> Element {
    debug!("rendering");
    let state_read = props.state.read();
    let (position, channel_id) = match *state_read {
        State::Idle => return rsx!(),
        State::Visible {
            position,
            channel_id,
        } => (position, channel_id),
    };

    let context_menu_handlers = use_context::<Handlers>();

    let menu_width = 160;
    let menu_height = 36;

    rsx! {
        g { transform: "translate({position.x} {position.y})",
            rect {
                x: "0",
                y: "0",
                width: "{menu_width}",
                height: "{menu_height}",
                fill: "white",
                stroke: "#ccc",
                style: "cursor: pointer;",
                onclick: move |evt| {
                    evt.stop_propagation();
                    context_menu_handlers.on_delete.call(channel_id);
                    context_menu_handlers.on_close.call(());
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

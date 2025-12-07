use beetry_core::MessageHash;
use beetry_serde::ser::channel::MessageSpec;
use dioxus::prelude::*;

use crate::definitions::{IndexedDragOffset, NodeId, Point};
use crate::ui::channel::temporary::ConnectionOrigin;
use crate::ui::text::{self, text_width_from};

#[derive(Debug, Clone)]
pub struct Handlers {
    on_mouse_down: EventHandler<(ConnectionOrigin, IndexedDragOffset, MessageHash)>,
    on_context_menu: EventHandler<(NodeId, MessageHash)>,
}

impl Handlers {
    pub(crate) fn new(
        on_mouse_down: impl FnMut((ConnectionOrigin, IndexedDragOffset, MessageHash)) + 'static,
        on_context_menu: impl FnMut((NodeId, MessageHash)) + 'static,
    ) -> Self {
        Self {
            on_mouse_down: EventHandler::new(on_mouse_down),
            on_context_menu: EventHandler::new(on_context_menu),
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ReceiverProps {
    id: NodeId,
    position: Point,
    spec: MessageSpec,
    channel_idx: usize,
    is_external: bool,
}

#[component]
pub fn Receiver(props: ReceiverProps) -> Element {
    let position = props.position;
    let node_id = props.id;
    let message_hash = props.spec.hash();

    let mut is_hovered = use_signal(|| false);
    let (fill_gradient, shadow_filter) = match (props.is_external, *is_hovered.peek()) {
        (true, true) => ("url(#channel-sender-gradient-hover)", "url(#shadow-hover)"),
        (true, false) => ("url(#channel-sender-gradient)", "url(#shadow)"),
        (false, true) => (
            "url(#channel-receiver-gradient-hover)",
            "url(#channel-receiver-shadow-hover)",
        ),
        (false, false) => ("url(#channel-receiver-gradient)", "url(#shadow)"),
    };
    let font_size = 10;
    let port_width = text_width_from(props.spec.as_str(), font_size);

    rsx! {
        g {
            rect {
                x: "{position.x + 80.0 - port_width}",
                y: "{position.y + 20.0 * props.channel_idx as f64}",
                width: "{port_width}",
                height: "20",
                rx: "4",
                ry: "4",
                fill: "{fill_gradient}",
                filter: "{shadow_filter}",
                stroke: "rgba(255,255,255,0.2)",
                stroke_width: "1",
                onmouseenter: move |_| is_hovered.set(true),
                onmouseleave: move |_| is_hovered.set(false),
                onmousedown: move |evt| {
                    evt.stop_propagation();
                    let mouse_coords = evt.element_coordinates();
                    let offset = Point {
                        x: mouse_coords.x,
                        y: mouse_coords.y,
                    };
                    use_context::<Handlers>()
                        .on_mouse_down
                        .call((
                            ConnectionOrigin::Receiver,
                            IndexedDragOffset {
                                id: props.id,
                                offset,
                            },
                            message_hash,
                        ))
                },
                oncontextmenu: move |evt| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    use_context::<Handlers>().on_context_menu.call((node_id, message_hash))
                },
            }
            text {
                x: "{position.x + 80.0 - (port_width / 2.0)}",
                y: "{position.y + 13.0 + 20.0 * props.channel_idx as f64}",
                fill: "white",
                font_family: text::font_family(),
                font_size: "{font_size}",
                font_weight: "medium",
                text_anchor: "middle",
                pointer_events: "none",
                "{props.spec.as_str()}"
            }
        }
    }
}

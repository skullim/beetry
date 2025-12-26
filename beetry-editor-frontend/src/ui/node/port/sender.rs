use beetry_editor_types::spec::message::MessageSpec;
use beetry_editor_types::{id::NodeId, id::NodePortId};
use dioxus::prelude::*;

use crate::definitions::{IndexedDragOffset, Point};
use crate::ui::channel::temporary::ConnectionOrigin;
use crate::ui::text::{self, text_width_from};

#[derive(Debug, Clone)]
pub struct Handlers {
    on_mouse_down: EventHandler<(ConnectionOrigin, IndexedDragOffset, NodePortId)>,
}

impl Handlers {
    pub(crate) fn new(
        on_mouse_down: impl FnMut((ConnectionOrigin, IndexedDragOffset, NodePortId)) + 'static,
    ) -> Self {
        Self {
            on_mouse_down: EventHandler::new(on_mouse_down),
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct SenderProps {
    id: NodeId,
    position: Point,
    msg_spec: MessageSpec,
    port_id: NodePortId,
}

#[component]
pub fn Sender(props: SenderProps) -> Element {
    let position = props.position;
    let message_desc = props.msg_spec.as_str();

    let mut is_hovered = use_signal(|| false);
    let (fill_gradient, shadow_filter) = if *is_hovered.peek() {
        ("url(#channel-sender-gradient-hover)", "url(#shadow-hover)")
    } else {
        ("url(#channel-sender-gradient)", "url(#shadow)")
    };

    let font_size = 10;
    let port_width = text_width_from(message_desc, font_size);

    let port_id = props.port_id;
    let port_id_as_f64 = port_id.raw_value() as f64;

    rsx! {
        g {
            rect {
                x: "{position.x}",
                y: "{position.y + 20.0 * port_id_as_f64}",
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
                            ConnectionOrigin::Sender,
                            IndexedDragOffset {
                                id: props.id,
                                offset,
                            },
                            port_id,
                        ))
                },
            }
            text {
                x: "{position.x + (port_width / 2.0)}",
                y: "{position.y + 13.0 + 20.0 * port_id_as_f64}",
                fill: "white",
                font_family: text::font_family(),
                font_size: "{font_size}",
                font_weight: "medium",
                text_anchor: "middle",
                pointer_events: "none",
                "{message_desc}"
            }
        }
    }
}

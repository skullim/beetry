use crate::Point;
use crate::ui::node::port::popup;
use crate::ui::{channel, shadow};
use beetry_editor_types::spec::message::MessageSpec;
use beetry_editor_types::{id::NodeId, id::NodePortId};
use dioxus::prelude::*;

use crate::definitions::IndexedDragOffset;
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
pub struct ReceiverProps {
    id: NodeId,
    position: Point,
    msg_spec: MessageSpec,
    port_id: NodePortId,
}

#[component]
pub fn Receiver(props: ReceiverProps) -> Element {
    debug!("rendering receiver port");

    let position = props.position;
    let node_id = props.id;
    let message_desc = props.msg_spec.as_str();

    static FONT_SIZE: u8 = 10;
    let port_width = text_width_from(message_desc, FONT_SIZE);

    let port_id = props.port_id;
    let port_id_as_f64 = port_id.raw_value() as f64;

    let mut is_hovered = use_signal(|| false);
    let is_external = use_signal(|| false);

    let mut port_popup_state = use_signal(popup::State::default);

    let fill = match (is_hovered(), is_external()) {
        (true, true) => channel::GradientHoverUrl::RECEIVER_EXTERNAL,
        (true, false) => channel::GradientHoverUrl::RECEIVER,
        (false, true) => channel::GradientUrl::RECEIVER_EXTERNAL,
        (false, false) => channel::GradientUrl::RECEIVER,
    };

    rsx! {
        g {
            rect {
                x: "{position.x + 80.0 - port_width}",
                y: "{position.y + 20.0 * port_id_as_f64}",
                width: "{port_width}",
                height: "20",
                rx: "4",
                ry: "4",
                fill,
                filter: if *is_hovered.read() { shadow::FilterUrl::SHADOW_HOVER } else { shadow::FilterUrl::SHADOW },
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
                            port_id,
                        ))
                },
                oncontextmenu: move |evt| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    port_popup_state
                        .set(popup::State::Visible {
                            id: node_id,
                            port_id,
                        });
                },
            }
            text {
                x: "{position.x + 80.0 - (port_width / 2.0)}",
                y: "{position.y + 13.0 + 20.0 * port_id_as_f64}",
                fill: "white",
                font_family: text::font_family(),
                font_size: "{FONT_SIZE}",
                font_weight: "medium",
                text_anchor: "middle",
                pointer_events: "none",
                "{message_desc}"
            }
        }
        popup::PortSettingsPopup { state: port_popup_state, is_external }
    }
}

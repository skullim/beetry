use crate::Point;
use crate::definitions::IndexedDragOffset;
use crate::Backend;
use crate::ui::handler::define_handlers;
use crate::ui::node::port::ConnectionOrigin;
use crate::ui::text::{self, text_width_from};
use crate::ui::{channel, shadow};
use beetry_editor_types::spec::message::MessageSpec;
use beetry_editor_types::{id::NodeId, id::NodePortId};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

define_handlers!(on_mouse_down: (ConnectionOrigin, IndexedDragOffset, NodePortId),
          on_menu: (Point, NodeId, NodePortId),
);

#[derive(Props, PartialEq, Clone)]
pub struct SenderProps {
    id: NodeId,
    position: Point,
    msg_spec: MessageSpec,
    port_id: NodePortId,
}

#[component]
pub fn Sender(props: SenderProps) -> Element {
    debug!("rendering");

    let position = props.position;
    let node_id = props.id;
    let message_desc = props.msg_spec.as_str();

    static FONT_SIZE: u8 = 10;
    let port_width = text_width_from(message_desc, FONT_SIZE);

    let port_id = props.port_id;
    let port_id_as_f64 = port_id.raw_value() as f64;
    let backend = use_context::<Backend>();
    let is_external = backend
        .with(|s| beetry_editor_backend::api::node::ports::is_external(s, node_id, port_id))
        .unwrap_or(false);

    let mut is_hovered = use_signal(|| false);
    let fill = match (is_hovered(), is_external) {
        (true, true) => channel::GradientHoverUrl::SENDER_EXTERNAL,
        (true, false) => channel::GradientHoverUrl::SENDER,
        (false, true) => channel::GradientUrl::SENDER_EXTERNAL,
        (false, false) => channel::GradientUrl::SENDER,
    };

    let handlers = use_context::<Handlers>();

    rsx! {
        g {
            rect {
                x: "{position.x}",
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
                    if evt.held_buttons().contains(MouseButton::Primary) && !is_external {
                        let mouse_coords = evt.element_coordinates();
                        let offset = Point {
                            x: mouse_coords.x,
                            y: mouse_coords.y,
                        };
                        handlers
                            .on_mouse_down
                            .call((
                                ConnectionOrigin::Sender,
                                IndexedDragOffset {
                                    id: props.id,
                                    offset,
                                },
                                port_id,
                            ))
                    }
                },
                oncontextmenu: move |evt| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    let click_point = Point {
                        x: evt.element_coordinates().x,
                        y: evt.element_coordinates().y,
                    };
                    handlers
                        .on_menu
                        .call((click_point, node_id, port_id))
                },
            }
            text {
                x: "{position.x + (port_width / 2.0)}",
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
    }
}

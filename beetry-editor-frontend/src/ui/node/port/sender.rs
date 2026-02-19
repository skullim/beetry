use std::rc::Rc;

use crate::Backend;
use crate::Point;
use crate::definitions::IndexedDragOffset;
use crate::ui::handler::define_handlers;
use crate::ui::node::port::{ConnectionOrigin, layout};
use crate::ui::text::{self, text_width_from};
use crate::ui::{channel, shadow};
use beetry_editor_backend::api::SpecByNodeIdQueryView;
use beetry_editor_types::{id::NodeId, id::NodePortId};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

define_handlers!(on_mouse_down: (ConnectionOrigin, IndexedDragOffset, NodePortId),
          on_menu: (Point, NodeId, NodePortId),
);

#[derive(Props, PartialEq, Clone)]
pub struct SenderProps {
    id: NodeId,
    port_id: NodePortId,
    position: Point,
}

#[component]
pub fn Sender(props: SenderProps) -> Element {
    debug!("rendering");

    let position = props.position;
    let node_id = props.id;
    let port_id = props.port_id;

    let backend = use_context::<Backend>();
    let message_desc = use_hook(|| {
        backend.with(|s| {
            let query_api = beetry_editor_backend::api::node::spec::by_node_id(s);
            let spec = query_api.ports(node_id).unwrap();
            let msg_spec = spec.spec(port_id).unwrap().msg_spec.desc().clone();
            Rc::new(msg_spec)
        })
    });

    let font_size = text::FONT_SIZE_SMALL;
    let port_width = text_width_from(&message_desc, font_size);

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
                y: "{position.y}",
                width: "{port_width}",
                height: "{layout::HEIGHT}",
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
                    handlers.on_menu.call((click_point, node_id, port_id))
                },
            }
            text {
                x: "{position.x + (port_width / 2.0)}",
                y: "{position.y + layout::TEXT_BASELINE_OFFSET}",
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

use beetry_editor_backend::api;
use std::rc::Rc;

use crate::Backend;
use crate::Point;
use crate::definitions::IndexedDragOffset;
use crate::signals::RenderRequests;
use crate::ui::handler::define_handlers;
use crate::ui::node::port::{ConnectionOrigin, layout};
use crate::ui::text::{self, text_width_from};
use crate::ui::{channel, shadow};
use beetry_editor_backend::api::SpecByNodeIdQueryView;
use beetry_editor_types::{id::NodeId, id::NodePortId};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

define_handlers!(on_mouse_down: (ConnectionOrigin, IndexedDragOffset, NodePortId),
                 on_menu: (Point, NodeId, NodePortId, Signal<bool>),
);

#[derive(Props, PartialEq, Clone)]
pub struct BodyProps {
    id: NodeId,
    port_id: NodePortId,
    position: Point,
    origin: ConnectionOrigin,
}

#[component]
pub fn Body(props: BodyProps) -> Element {
    debug!(
        "rendering (node id: {}, port id: {})",
        props.id, props.port_id
    );
    let position = props.position;
    let node_id = props.id;
    let port_id = props.port_id;
    let origin = props.origin;

    let backend = use_context::<Backend>();
    let message_desc = use_hook(|| {
        backend.with(|s| {
            let query_api = api::node::spec::by_node_id(s);
            let spec = query_api.ports(node_id).unwrap();
            let msg_spec = spec.spec(port_id).unwrap().msg_spec.desc().clone();
            Rc::new(msg_spec)
        })
    });

    let font_size = text::FONT_SIZE_SMALL;
    let port_width = text_width_from(&message_desc, font_size);

    let is_external = use_signal(|| {
        backend
            .with(|s| api::node::ports::is_external(s, node_id, port_id))
            .unwrap_or(false)
    });
    let mut is_connected = use_signal(|| {
        backend.with(|s| {
            api::node::ports::connection_views(s)
                .filter_map(Result::ok)
                .any(|conn| conn.node_id == node_id && conn.port_id == port_id)
        })
    });
    let mut is_hovered = use_signal(|| false);
    let render_requests = use_context::<RenderRequests>();

    use_memo(move || {
        render_requests.channel_edges.track();
        let connected = backend.with(|s| {
            api::node::ports::connection_views(s)
                .filter_map(Result::ok)
                .any(|conn| conn.node_id == node_id && conn.port_id == port_id)
        });
        is_connected.set(connected);
    });

    let fill = match (origin, is_hovered(), is_external(), is_connected()) {
        (ConnectionOrigin::Sender, true, false, false) => {
            channel::GradientHoverUrl::DISCONNECTED_SENDER
        }
        (ConnectionOrigin::Sender, false, false, false) => {
            channel::GradientUrl::DISCONNECTED_SENDER
        }
        (ConnectionOrigin::Receiver, true, false, false) => {
            channel::GradientHoverUrl::DISCONNECTED_RECEIVER
        }
        (ConnectionOrigin::Receiver, false, false, false) => {
            channel::GradientUrl::DISCONNECTED_RECEIVER
        }

        (ConnectionOrigin::Sender, true, true, _) => channel::GradientHoverUrl::SENDER_EXTERNAL,
        (ConnectionOrigin::Sender, false, true, _) => channel::GradientUrl::SENDER_EXTERNAL,
        (ConnectionOrigin::Receiver, true, true, _) => channel::GradientHoverUrl::RECEIVER_EXTERNAL,
        (ConnectionOrigin::Receiver, false, true, _) => channel::GradientUrl::RECEIVER_EXTERNAL,

        (ConnectionOrigin::Sender, true, false, true) => channel::GradientHoverUrl::SENDER,
        (ConnectionOrigin::Sender, false, false, true) => channel::GradientUrl::SENDER,
        (ConnectionOrigin::Receiver, true, false, true) => channel::GradientHoverUrl::RECEIVER,
        (ConnectionOrigin::Receiver, false, false, true) => channel::GradientUrl::RECEIVER,
    };

    let (x, text_x) = match origin {
        ConnectionOrigin::Receiver => (position.x - port_width, position.x - (port_width / 2.0)),
        ConnectionOrigin::Sender => (position.x, position.x + (port_width / 2.0)),
    };

    let handlers = use_context::<Handlers>();

    rsx! {
        g {
            rect {
                x: "{x}",
                y: "{position.y}",
                width: "{port_width}",
                height: "{layout::HEIGHT}",
                rx: "4",
                ry: "4",
                fill,
                filter: if is_hovered() { shadow::FilterUrl::SHADOW_HOVER } else { shadow::FilterUrl::SHADOW },
                stroke: "rgba(255,255,255,0.2)",
                stroke_width: "1",
                onmouseenter: move |_| is_hovered.set(true),
                onmouseleave: move |_| is_hovered.set(false),
                onmousedown: move |evt| {
                    evt.stop_propagation();
                    if evt.held_buttons().contains(MouseButton::Primary) && !*is_external.peek() {
                        let mouse_coords = evt.element_coordinates();
                        let offset = Point {
                            x: mouse_coords.x,
                            y: mouse_coords.y,
                        };
                        handlers
                            .on_mouse_down
                            .call((
                                origin,
                                IndexedDragOffset {
                                    id: props.id,
                                    offset,
                                },
                                port_id,
                            ));
                    }
                },
                oncontextmenu: move |evt| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    let click_point = Point {
                        x: evt.element_coordinates().x,
                        y: evt.element_coordinates().y,
                    };
                    handlers.on_menu.call((click_point, node_id, port_id, is_external));
                },
            }
            text {
                class: "bt-text-sm",
                x: "{text_x}",
                y: "{position.y + layout::TEXT_BASELINE_OFFSET}",
                fill: "white",
                font_weight: "medium",
                text_anchor: "middle",
                pointer_events: "none",
                "{message_desc}"
            }
        }
    }
}

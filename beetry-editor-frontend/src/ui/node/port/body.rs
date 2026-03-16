use std::rc::Rc;

use beetry_editor_backend::{
    api,
    api::SpecByNodeIdQuery,
    node::{PortConnectionQuery, PortStateQuery},
};
use beetry_editor_types::{
    id::{NodeId, NodePortId},
    output::node::PortState,
};
use dioxus::{html::input_data::MouseButton, prelude::*};

use crate::{
    Backend, Point,
    definitions::IndexedDragOffset,
    signals::RenderRequests,
    ui::{
        handler::define_handlers,
        node::port::{ConnectionOrigin, layout},
        style::{
            connection::{Fill, FillHover, Stroke},
            shadow,
            text::{self, text_width_from},
        },
    },
};

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
        backend.with(|s| {
            let query = api::node::ports::state_query(s);
            query
                .state(node_id, port_id)
                .map(PortState::is_external)
                .unwrap_or(false)
        })
    });
    let mut is_connected = use_signal(|| {
        backend.with(|s| api::node::ports::connections_query(s).is_port_connected(node_id, port_id))
    });
    let mut is_hovered = use_signal(|| false);
    let render_requests = use_context::<RenderRequests>();

    use_memo(move || {
        render_requests.channel_edges.track();
        let connected = backend
            .with(|s| api::node::ports::connections_query(s).is_port_connected(node_id, port_id));
        is_connected.set(connected);
    });

    let fill = match (origin, is_hovered(), is_external(), is_connected()) {
        (_, _, false, false) => Fill::UNCONNECTED,
        (_, true, true, _) => FillHover::EXTERNAL_PORT,
        (_, false, true, _) => Fill::EXTERNAL_PORT,

        (ConnectionOrigin::Sender, true, false, true) => FillHover::SENDER,
        (ConnectionOrigin::Sender, false, false, true) => Fill::SENDER,
        (ConnectionOrigin::Receiver, true, false, true) => FillHover::RECEIVER,
        (ConnectionOrigin::Receiver, false, false, true) => Fill::RECEIVER,
    };
    let stroke = match (is_external(), is_connected(), is_hovered()) {
        (false, false, true) => Stroke::UNCONNECTED_HOVER,
        (false, false, false) => Stroke::UNCONNECTED_DEFAULT,
        _ => Stroke::DEFAULT,
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
                stroke,
                stroke_width: Stroke::WIDTH,
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

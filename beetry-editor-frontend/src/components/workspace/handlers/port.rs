use beetry_editor_types::{
    id::{NodeId, NodePortId},
    output::edge::NodeEdge,
    output::ui::Point,
};
use dioxus::logger::tracing::debug;
use dioxus::prelude::*;

use super::{Backend, RenderRequests};
use crate::components::workspace::state::{menu, temporary};
use crate::definitions::{EdgePos, IndexedDragOffset};
use crate::ui::error::ErrorQueueState;
use crate::ui::node::port::ConnectionOrigin;
use crate::ui::node::{self};

pub(crate) fn input_handlers(
    mut temp: temporary::State,
    mut backend: Backend,
    mut requests: RenderRequests,
) -> node::port::input::Handlers {
    let on_mouse_up = move |to: NodeId| -> Result<()> {
        let mut edge = temp.edge.write();
        if let Some(from) = edge.take_dragged()
            && from != to
        {
            backend
                .with_mut(|s| beetry_editor_backend::api::edge::create(s, NodeEdge { from, to }))?;
            requests.edges.request();
            debug!("created edge from node {from}: to: {to}");
        }
        Ok(())
    };
    node::port::input::Handlers::new(on_mouse_up)
}

pub(crate) fn output_handlers(mut temp: temporary::State) -> node::port::output::Handlers {
    let on_mouse_down = move |indexed_drag_offset: IndexedDragOffset| {
        let (id, offset) = (indexed_drag_offset.id, indexed_drag_offset.offset);
        temp.edge.with_mut(|e| {
            e.set_dragged(
                id,
                EdgePos {
                    start: offset,
                    end: offset,
                },
            );
        });
        Ok(())
    };
    node::port::output::Handlers::new(on_mouse_down)
}

pub(crate) fn sender_handlers(
    mut menu: menu::State,
    mut temp: temporary::State,
    backend: Backend,
    mut errors: ErrorQueueState,
) -> node::port::sender::Handlers {
    let on_mouse_down = move |(origin, indexed_drag_offset, port_id): (
        ConnectionOrigin,
        IndexedDragOffset,
        NodePortId,
    )| {
        let dragged_data = temporary::channel_edge::DraggedData {
            node_id: indexed_drag_offset.id,
            origin,
            port_id,
        };
        let offset = indexed_drag_offset.offset;
        temp.channel.with_mut(|c| {
            c.set_dragged(
                dragged_data,
                EdgePos {
                    start: offset,
                    end: offset,
                },
            );
        });
        Ok(())
    };
    let on_menu = move |(position, node_id, port_id): (Point, NodeId, NodePortId)| {
        let is_external = match backend
            .with(|s| beetry_editor_backend::api::node::ports::is_external(s, node_id, port_id))
        {
            Ok(value) => value,
            Err(err) => {
                errors.with_mut(|q| q.push("port-menu", err.to_string()));
                false
            }
        };
        menu.port.set(node::port::menu::State::Visible {
            position,
            id: node_id,
            port_id,
            is_external,
        });
        Ok(())
    };

    node::port::sender::Handlers::new(on_mouse_down, on_menu)
}

pub(crate) fn receiver_handlers(
    mut menu: menu::State,
    mut temp: temporary::State,
    backend: Backend,
    mut errors: ErrorQueueState,
) -> node::port::receiver::Handlers {
    let on_mouse_down = move |(origin, indexed_drag_offset, port_id): (
        ConnectionOrigin,
        IndexedDragOffset,
        NodePortId,
    )| {
        let dragged_data = temporary::channel_edge::DraggedData {
            node_id: indexed_drag_offset.id,
            origin,
            port_id,
        };
        let offset = indexed_drag_offset.offset;
        temp.channel.with_mut(|c| {
            c.set_dragged(
                dragged_data,
                EdgePos {
                    start: offset,
                    end: offset,
                },
            );
        });
        Ok(())
    };
    let on_menu = move |(position, node_id, port_id): (Point, NodeId, NodePortId)| {
        let is_external = match backend
            .with(|s| beetry_editor_backend::api::node::ports::is_external(s, node_id, port_id))
        {
            Ok(value) => value,
            Err(err) => {
                errors.with_mut(|q| q.push("port-menu", err.to_string()));
                false
            }
        };
        menu.port.set(node::port::menu::State::Visible {
            position,
            id: node_id,
            port_id,
            is_external,
        });
        Ok(())
    };

    node::port::receiver::Handlers::new(on_mouse_down, on_menu)
}

pub(crate) fn menu_handlers(
    mut backend: Backend,
    mut requests: RenderRequests,
) -> node::port::menu::Handlers {
    let on_external = move |(node_id, port_id): (NodeId, NodePortId)| {
        backend
            .with_mut(|s| {
                beetry_editor_backend::api::node::ports::set_external(s, node_id, port_id)
            })
            .unwrap();
        requests.ports.request();
        debug!("set port (node id: {node_id}, port id: {port_id}) as external")
    };

    let on_internal = move |(node_id, port_id): (NodeId, NodePortId)| {
        backend
            .with_mut(|s| {
                beetry_editor_backend::api::node::ports::set_internal(s, node_id, port_id)
            })
            .unwrap();
        requests.ports.request();
        debug!("set port (node id: {node_id}, port id: {port_id}) as internal")
    };

    node::port::menu::Handlers::builder()
        .on_external(on_external)
        .on_internal(on_internal)
        .build()
}

use beetry_editor_types::{
    id::{NodeId, NodePortId},
    output::edge::NodeEdge,
    output::ui::Point,
};
use dioxus::logger::tracing::debug;
use dioxus::prelude::*;

use super::{Backend, MenuState, RenderRequests, TempState};
use crate::definitions::{EdgePos, IndexedDragOffset};
use crate::ui::channel::temporary::{ConnectionOrigin, DraggedData};
use crate::ui::error_dialog::ErrorQueueState;
use crate::ui::node::{self, ReceiverPortHandlers, SenderPortHandlers};

pub(crate) fn input_handlers(
    mut temp: TempState,
    mut backend: Backend,
    mut requests: RenderRequests,
) -> node::InputPortHandlers {
    let on_mouse_up = move |to: NodeId| -> Result<()> {
        if let Some(from) = temp.edge.take_dragged()
            && from != to
        {
            backend
                .with_mut(|s| beetry_editor_backend::api::edge::create(s, NodeEdge { from, to }))?;
            requests.edges.request();
            debug!("created edge from node {from}: to: {to}");
        }
        Ok(())
    };
    node::InputPortHandlers::new(on_mouse_up)
}

pub(crate) fn output_handlers(mut temp: TempState) -> node::OutputPortHandlers {
    let on_mouse_down = move |indexed_drag_offset: IndexedDragOffset| {
        let offset = indexed_drag_offset.offset;
        temp.edge.set_dragged_from(indexed_drag_offset.id);
        temp.edge.update_edge(EdgePos {
            start: offset,
            end: offset,
        });
        Ok(())
    };
    node::OutputPortHandlers::new(on_mouse_down)
}

pub(crate) fn sender_handlers(
    mut menus: MenuState,
    mut temp: TempState,
    backend: Backend,
    mut errors: ErrorQueueState,
) -> SenderPortHandlers {
    let on_mouse_down = move |(origin, indexed_drag_offset, port_id): (
        ConnectionOrigin,
        IndexedDragOffset,
        NodePortId,
    )| {
        let dragged_data = DraggedData {
            node_id: indexed_drag_offset.id,
            origin,
            port_id,
        };
        let offset = indexed_drag_offset.offset;
        temp.channel.set_dragged(dragged_data);
        temp.channel.update_edge_pos(EdgePos {
            start: offset,
            end: offset,
        });
        Ok(())
    };
    let on_context_menu = move |(position, node_id, port_id): (Point, NodeId, NodePortId)| {
        let is_external = match backend
            .with(|s| beetry_editor_backend::api::node::ports::is_external(s, node_id, port_id))
        {
            Ok(value) => value,
            Err(err) => {
                errors.with_mut(|q| q.push("port-context-menu", err.to_string()));
                false
            }
        };
        menus.port.set(node::port_context_menu::State::Visible {
            position,
            id: node_id,
            port_id,
            is_external,
        });
        Ok(())
    };

    SenderPortHandlers::new(on_mouse_down, on_context_menu)
}

pub(crate) fn receiver_handlers(
    mut menus: MenuState,
    mut temp: TempState,
    backend: Backend,
    mut errors: ErrorQueueState,
) -> ReceiverPortHandlers {
    let on_mouse_down = move |(origin, indexed_drag_offset, port_id): (
        ConnectionOrigin,
        IndexedDragOffset,
        NodePortId,
    )| {
        let dragged_data = DraggedData {
            node_id: indexed_drag_offset.id,
            origin,
            port_id,
        };
        let offset = indexed_drag_offset.offset;
        temp.channel.set_dragged(dragged_data);
        temp.channel.update_edge_pos(EdgePos {
            start: offset,
            end: offset,
        });
        Ok(())
    };
    let on_context_menu = move |(position, node_id, port_id): (Point, NodeId, NodePortId)| {
        let is_external = match backend
            .with(|s| beetry_editor_backend::api::node::ports::is_external(s, node_id, port_id))
        {
            Ok(value) => value,
            Err(err) => {
                errors.with_mut(|q| q.push("port-context-menu", err.to_string()));
                false
            }
        };
        menus.port.set(node::port_context_menu::State::Visible {
            position,
            id: node_id,
            port_id,
            is_external,
        });
        Ok(())
    };

    ReceiverPortHandlers::new(on_mouse_down, on_context_menu)
}

pub(crate) fn context_menu_handlers(
    mut backend: Backend,
    mut requests: RenderRequests,
) -> node::PortContextMenuHandlers {
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

    node::PortContextMenuHandlers::builder()
        .on_external(on_external)
        .on_internal(on_internal)
        .build()
}

use beetry_editor_backend::api;
use beetry_editor_types::id::ChannelEdgeId;
use dioxus::prelude::*;

use super::{Backend, RenderRequests};
use crate::components::workspace::state::menu;
use crate::ui::channel::{edge, edge_menu};

pub(crate) fn handlers(mut menu: menu::State) -> edge::Handlers {
    let on_menu = move |(edge, position): (ChannelEdgeId, crate::Point)| {
        menu.channel_edge.set(edge_menu::State::Visible {
            position,
            edge,
        });
        Ok(())
    };

    edge::Handlers::new(on_menu)
}

pub(crate) fn menu_handlers(
    mut menu: menu::State,
    mut backend: Backend,
    mut requests: RenderRequests,
) -> edge_menu::Handlers {
    let on_delete = move |edge: ChannelEdgeId| -> Result<()> {
        backend.with_mut(|s| {
            api::node::ports::disconnect(
                s,
                edge.node_id,
                edge.port_id,
                edge.channel_id,
            )
        })?;
        requests.channel_edges.request();
        Ok(())
    };

    let on_close = move |()| {
        menu.channel_edge.set(edge_menu::State::Idle);
        Ok(())
    };

    edge_menu::Handlers::new(on_delete, on_close)
}

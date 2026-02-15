use beetry_editor_types::id::EdgeId;
use dioxus::prelude::*;

use super::{Backend, MenuState, RenderRequests};
use crate::ui::edge;

pub(crate) fn handlers(mut menus: MenuState) -> edge::Handlers {
    let on_context_menu = move |(edge_id, position): (EdgeId, crate::Point)| {
        menus
            .edge
            .set(edge::ContextMenuState::Visible { position, edge_id });
        Ok(())
    };
    edge::Handlers::new(on_context_menu)
}

pub(crate) fn context_menu_handlers(
    mut menus: MenuState,
    mut backend: Backend,
    mut requests: RenderRequests,
) -> edge::ContextMenuHandlers {
    let on_delete = move |id: EdgeId| -> Result<()> {
        backend.with_mut(|s| beetry_editor_backend::api::edge::remove(s, id))?;
        requests.edges.request();
        Ok(())
    };

    let on_close = move |()| {
        menus.edge.set(edge::ContextMenuState::Idle);
        Ok(())
    };

    edge::ContextMenuHandlers::new(on_delete, on_close)
}

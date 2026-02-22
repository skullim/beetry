use beetry_editor_backend::api;
use beetry_editor_types::id::EdgeId;
use dioxus::prelude::*;

use super::{Backend, RenderRequests};
use crate::{components::workspace::state::menu, ui::edge};

pub(crate) fn handlers(mut menu: menu::State) -> edge::Handlers {
    let on_menu = move |(edge_id, position): (EdgeId, crate::Point)| {
        menu.edge
            .set(edge::menu::State::Visible { position, edge_id });
        Ok(())
    };
    edge::Handlers::new(on_menu)
}

pub(crate) fn menu_handlers(
    mut menu: menu::State,
    mut backend: Backend,
    mut requests: RenderRequests,
) -> edge::menu::Handlers {
    let on_delete = move |id: EdgeId| -> Result<()> {
        backend.with_mut(|s| api::edge::remove(s, id))?;
        requests.edges.request();
        Ok(())
    };

    let on_close = move |()| {
        menu.edge.set(edge::menu::State::Idle);
        Ok(())
    };

    edge::menu::Handlers::new(on_delete, on_close)
}

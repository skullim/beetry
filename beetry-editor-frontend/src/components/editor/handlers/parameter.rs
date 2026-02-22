use beetry_editor_backend::api;
use beetry_editor_types::{id::NodeId, output::node::Parameters};
use dioxus::prelude::*;

use crate::Backend;
use crate::signals::RequestNodeRender;
use crate::ui::node;

pub(crate) fn handlers(
    mut backend: Backend,
    mut state: Signal<node::parameter::State>,
    mut render_nodes: RequestNodeRender,
) -> node::parameter::Handlers {
    let on_confirm = move |(node_id, params): (NodeId, Parameters)| {
        backend.with_mut(|s| api::node::parameters::create(s, node_id, params));
        state.take();
        Ok(())
    };

    let on_cancel = move |_| -> Result<()> {
        let state = state.take();
        if let node::parameter::State::Visible { id, mode, .. } = state
            && mode == node::parameter::Mode::Create
        {
            backend.with_mut(|s| api::node::remove(s, id))?;
            render_nodes.request();
        }
        Ok(())
    };

    node::parameter::Handlers::new(on_confirm, on_cancel)
}

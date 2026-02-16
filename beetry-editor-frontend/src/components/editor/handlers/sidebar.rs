use beetry_editor_types::{output::ui::NodeUiData, spec::node::NodeSpecKey};
use dioxus::prelude::*;

use super::super::state;
use crate::signals::RequestNodeRender;
use crate::ui::node::PARAMETER_POSITION;
use crate::{Backend, SharedSpecs, sidebar, ui::node};

pub(crate) fn handlers(
    specs: SharedSpecs,
    mut request: RequestNodeRender,
    mut backend: Backend,
    mut state: state::State,
) -> sidebar::Handlers {
    let on_new_node = move |node_spec_key: NodeSpecKey| -> Result<()> {
        let node_spec = specs.nodes.spec(&node_spec_key)?;

        let ui_data = state
            .element_spawn_point
            .with_peek(|p| NodeUiData { position: *p });
        let id = backend
            .with_mut(|s| beetry_editor_backend::api::node::create(s, node_spec, ui_data))?;

        if node_spec.has_params() {
            state.parameter.set(node::parameter::State::Visible {
                position: PARAMETER_POSITION,
                id,
                mode: node::parameter::Mode::Create,
            });
        }

        info!(
            "created node {id} with name {} and {:?} kind",
            node_spec_key.name(),
            node_spec_key.kind()
        );
        request.request();
        Ok(())
    };

    sidebar::Handlers::new(on_new_node)
}

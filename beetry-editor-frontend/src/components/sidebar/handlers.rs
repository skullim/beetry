use beetry_editor_backend::api;
use beetry_editor_types::{
    output::ui::NodeUiData,
    spec::{channel::ChannelSpec, node::NodeSpecKey},
};
use dioxus::prelude::*;

use super::Handlers;
use crate::{
    Backend, SharedSpecs,
    components::editor,
    signals::RequestNodeRender,
    ui::{channel, node, node::parameter},
};

pub fn handlers(
    specs: SharedSpecs,
    mut request: RequestNodeRender,
    mut backend: Backend,
    mut state: editor::State,
) -> Handlers {
    let on_new_node = move |node_spec_key: NodeSpecKey| -> Result<()> {
        let node_spec = specs.nodes.spec(&node_spec_key)?;

        let ui_data = state
            .element_spawn_point
            .with_peek(|p| NodeUiData { position: *p });
        let id = backend.with_mut(|s| api::node::create(s, node_spec, ui_data))?;

        if node_spec.has_params() {
            state.parameter.set(node::parameter::State::Visible {
                position: parameter::DEFAULT_DIALOG_POSITION,
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

    let on_new_channel = move |spec: ChannelSpec| -> Result<()> {
        state.channel_dialog.set(channel::dialog::State::Visible {
            position: channel::dialog::DEFAULT_POSITION,
            mode: channel::dialog::Mode::Create {
                spec_key: spec.msg_hash(),
            },
            config: state.default_channel_config,
        });
        Ok(())
    };

    Handlers::new(on_new_node, on_new_channel)
}

use crate::signals::RenderRequests;
use crate::{Point, SharedSpecs};
use beetry_core::MessageHash;
use beetry_editor_backend::EditorService;
use beetry_editor_types::output::{
    channel::ChannelConfig,
    node::Parameters,
    ui::{ChannelUiData, NodeUiData},
};
use beetry_editor_types::{id::NodeId, spec::node::NodeSpecKey};
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::components::workspace::Workspace;
use crate::sidebar::Sidebar;
use crate::ui::error_dialog::ErrorQueueState;
use crate::ui::node;
use crate::ui::node::PARAM_DIALOG_POSITION;
use crate::{NodeSpecMap, toolbar::Toolbar};
use crate::{sidebar, ui::channel::config::State as ChannelConfigDialogState};
use crate::{
    signals::RequestNodeRender,
    ui::channel::{self},
};

#[derive(Clone, Copy)]
pub struct Backend {
    service: CopyValue<EditorService>,
}

impl std::ops::Deref for Backend {
    type Target = CopyValue<EditorService>;
    fn deref(&self) -> &Self::Target {
        &self.service
    }
}

impl std::ops::DerefMut for Backend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.service
    }
}

impl Backend {
    fn new(node_specs: NodeSpecMap) -> Self {
        Self {
            service: CopyValue::new(EditorService::new(node_specs)),
        }
    }
}

#[component]
pub(crate) fn Editor() -> Element {
    debug!("rendering");

    let specs = use_context::<SharedSpecs>();
    let mut backend = use_context_provider(|| Backend::new(specs.nodes.clone()));
    use_context_provider(ErrorQueueState::new);

    let mut channel_config_dialog_state: Signal<ChannelConfigDialogState> =
        use_signal(ChannelConfigDialogState::default);
    let mut parameter_dialog_state: Signal<node::parameter_dialog::State> =
        use_signal(node::parameter_dialog::State::default);
    let mut render_requests = use_context_provider(RenderRequests::default);
    let element_spawn_point = use_signal(Point::default);

    // sidebar event handlers
    use_context_provider(move || {
        let on_new_node = move |node_spec_key: NodeSpecKey| -> Result<()> {
            let node_spec = specs.nodes.spec(&node_spec_key)?;

            let spawn_point_read = element_spawn_point.read();
            let ui_data = NodeUiData {
                position: *spawn_point_read,
            };

            let id = backend
                .with_mut(|s| beetry_editor_backend::api::node::create(s, node_spec, ui_data))?;

            if node_spec.has_params() {
                parameter_dialog_state.set(node::parameter_dialog::State::Visible {
                    position: PARAM_DIALOG_POSITION,
                    id,
                    mode: node::parameter_dialog::Mode::Create,
                });
            }

            info!(
                "created node {id} with name {} and {:?} kind",
                node_spec_key.name(),
                node_spec_key.kind()
            );
            render_requests.nodes.request();
            Ok(())
        };

        sidebar::Handlers::new(on_new_node)
    });

    // behind an Rc, cheap clone
    let specs = use_context::<SharedSpecs>();
    use_context_provider(move || {
        let on_new_channel = move |(spec_key, config): (MessageHash, ChannelConfig)| -> Result<()> {
            let spec = specs.channels.spec(&spec_key)?;

            let spawn_point_read = element_spawn_point.read();
            let ui_data = ChannelUiData {
                position: *spawn_point_read,
            };

            let id = backend.with_mut(|s| {
                beetry_editor_backend::api::channel::create(s, spec, config, ui_data)
            })?;

            info!(
                "created channel {id} with message type {}",
                spec.msg_type_name()
            );
            channel_config_dialog_state.take();
            render_requests.channels.request();
            Ok(())
        };

        let on_cancel = move |_| {
            channel_config_dialog_state.take();
            Ok(())
        };
        channel::config::Handlers::new(on_new_channel, on_cancel)
    });

    use_context_provider(move || {
        parameter_dialog_handlers(backend, parameter_dialog_state, render_requests.nodes)
    });

    rsx! {
        div { style: "display: flex; flex-direction: row; gap: 10px;",
            div { style: "flex: 0 1 20%;",
                Sidebar { channel_config_dialog_state }
            }
            div { style: "flex: 0 1 80%;",
                Workspace {
                    render_requests,
                    element_spawn_point,
                    parameter_dialog_state,
                }
            }
            div {
                node::parameter_dialog::Dialog { state: parameter_dialog_state }
            }
            div { style: "flex: 0 1 10%;",
                Toolbar { render_requests }
            }
        }
    }
}

fn parameter_dialog_handlers(
    mut backend: Backend,
    mut state: Signal<node::parameter_dialog::State>,
    mut render_nodes: RequestNodeRender,
) -> node::parameter_dialog::Handlers {
    let on_confirm = move |(node_id, params): (NodeId, Parameters)| {
        backend
            .with_mut(|s| beetry_editor_backend::api::node::parameters::create(s, node_id, params));
        state.take();
        Ok(())
    };

    let on_cancel = move |_| -> Result<()> {
        let state = state.take();
        if let node::parameter_dialog::State::Visible { id, mode, .. } = state
            && mode == node::parameter_dialog::Mode::Create
        {
            backend.with_mut(|s| beetry_editor_backend::api::node::remove(s, id))?;
            render_nodes.request();
        }
        Ok(())
    };

    node::parameter_dialog::Handlers::new(on_confirm, on_cancel)
}

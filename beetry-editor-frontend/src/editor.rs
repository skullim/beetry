use crate::ui::node::ParameterDialog;
use crate::{Point, SharedSpecs};
use beetry_editor_backend::EditorService;
use beetry_editor_backend::repository::{
    ChannelRepositoryFacade, EdgeRepository, NodeRepositoryFacade, UiRepositoryFacade,
};
use beetry_editor_types::output::{
    channel::ChannelConfig,
    node::Parameters,
    ui::{ChannelUiData, NodeUiData},
};
use beetry_editor_types::{id::NodeId, spec::node::NodeSpecKey};
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::sidebar::Sidebar;
use crate::ui::node::{ParameterDialogHandlers, ParameterDialogState};
use crate::workspace::Workspace;
use crate::{NodeSpecMap, toolbar::Toolbar};
use crate::{
    sidebar::SidebarEventHandlers, ui::channel::config_dialog::State as ChannelConfigDialogState,
};
use crate::{
    signals::RequestRender,
    ui::channel::{self},
};

type EditorServiceImpl = EditorService<
    NodeRepositoryFacade,
    EdgeRepository,
    ChannelRepositoryFacade,
    UiRepositoryFacade,
>;

#[derive(Clone)]
pub struct ServiceContext {
    pub service: CopyValue<EditorServiceImpl>,
}

impl std::ops::Deref for ServiceContext {
    type Target = CopyValue<EditorServiceImpl>;
    fn deref(&self) -> &Self::Target {
        &self.service
    }
}

impl std::ops::DerefMut for ServiceContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.service
    }
}

impl ServiceContext {
    fn new(node_specs: NodeSpecMap) -> Self {
        Self {
            service: CopyValue::new(EditorServiceImpl::new(node_specs)),
        }
    }
}

#[component]
pub(crate) fn Editor() -> Element {
    debug!("rendering");

    let specs = use_context::<SharedSpecs>();
    use_context_provider(|| ServiceContext::new(specs.nodes.clone()));

    let mut channel_config_dialog_state: Signal<ChannelConfigDialogState> =
        use_signal(ChannelConfigDialogState::default);
    let mut parameter_dialog_state: Signal<ParameterDialogState> =
        use_signal(ParameterDialogState::default);

    let mut render_nodes = use_signal(RequestRender::new);
    let render_edges = use_signal(RequestRender::new);
    let mut render_channels = use_signal(RequestRender::new);

    let ui_spawn_point = use_signal(Point::default);

    use_context_provider(move || {
        let on_new_node = move |node_spec_key: NodeSpecKey| -> Result<()> {
            let specs = use_context::<SharedSpecs>();
            let node_spec = specs.nodes.spec(&node_spec_key)?;

            let spawn_point_read = ui_spawn_point.read();
            let ui_data = NodeUiData {
                position: *spawn_point_read,
            };

            let mut service = use_context::<ServiceContext>();
            let id = service.with_mut(|s| s.create_node(node_spec, ui_data))?;

            if node_spec.params().is_some() {
                parameter_dialog_state.set(ParameterDialogState::Visible {
                    position: Point { x: 300.0, y: 200.0 },
                    id,
                });
            }

            info!(
                "created node {id} with name {} and {:?} kind",
                node_spec_key.name(),
                node_spec_key.kind()
            );
            render_nodes.with_mut(|write| write.request());
            Ok(())
        };

        SidebarEventHandlers::new(on_new_node)
    });

    use_context_provider(move || {
        let on_new_channel = move |config: ChannelConfig| -> Result<()> {
            if let ChannelConfigDialogState::Visible { spec, .. } =
                channel_config_dialog_state.take()
            {
                let spawn_point_read = ui_spawn_point.read();
                let ui_data = ChannelUiData {
                    position: *spawn_point_read,
                };

                let mut service = use_context::<ServiceContext>();
                let id = service.with_mut(|s| s.create_channel(&spec, config, ui_data))?;

                info!(
                    "created channel {id} with message type {}",
                    spec.msg_type_name(),
                );
                render_channels.with_mut(|write| write.request());
            }
            Ok(())
        };

        let on_cancel = move |_| {
            channel_config_dialog_state.take();
        };
        channel::config_dialog::Handlers::new(on_new_channel, on_cancel)
    });

    use_context_provider(move || parameter_dialog_handlers(parameter_dialog_state, render_nodes));

    rsx! {
        div { style: "display: flex; flex-direction: row; gap: 10px;",
            div { style: "flex: 0 1 20%;",
                Sidebar { channel_config_dialog_state }
            }
            div { style: "flex: 0 1 80%;",
                Workspace {
                    render_nodes,
                    render_channels,
                    render_edges,
                    ui_spawn_point,
                }
            }
            div {
                ParameterDialog { state: parameter_dialog_state }
            }
            div { style: "flex: 0 1 10%;",
                Toolbar { render_nodes, render_channels, render_edges }
            }
        }
    }
}

fn parameter_dialog_handlers(
    mut state: Signal<ParameterDialogState>,
    mut render_nodes: Signal<RequestRender>,
) -> ParameterDialogHandlers {
    let on_confirm = move |(node_id, params): (NodeId, Parameters)| {
        let mut service_ctx = use_context::<ServiceContext>();
        service_ctx.with_mut(|s| s.node_api_mut().parameters().create(node_id, params));
        state.take();
    };

    let on_cancel = move |_| -> Result<()> {
        let state = state.take();
        if let ParameterDialogState::Visible { id, .. } = state {
            let mut service_ctx = use_context::<ServiceContext>();
            service_ctx.with_mut(|s| s.remove_node(id))?;
            render_nodes.with_mut(|write| write.request());
        }
        Ok(())
    };

    ParameterDialogHandlers::new(on_confirm, on_cancel)
}

use crate::Point;
use crate::ui::node::ParameterDialog;
use beetry_editor_backend::EditorService;
use beetry_editor_backend::repository::{
    ChannelRepositoryFacade, EdgeRepository, NodeRepositoryFacade, UiRepositoryFacade,
};
use beetry_editor_types::output::{
    channel::ChannelConfig,
    node::Parameters,
    ui::{ChannelUiData, NodeUiData},
};
use beetry_editor_types::{id::NodeId, persistence::ParameterValue, spec::node::NodeSpecKey};
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::ui::node::{ParameterDialogHandlers, ParameterDialogState};
use crate::workspace::Workspace;
use crate::{NodeSpecMap, toolbar::Toolbar};
use crate::{Specs, sidebar::Sidebar};
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

impl ServiceContext {
    fn new(node_specs: NodeSpecMap) -> Self {
        Self {
            service: CopyValue::new(EditorServiceImpl::new(node_specs)),
        }
    }
}

#[component]
pub(crate) fn Editor() -> Element {
    debug!("rendering editor");

    let specs = use_context::<Specs>();
    use_context_provider(|| ServiceContext::new(specs.nodes));

    let mut channel_config_dialog_state: Signal<ChannelConfigDialogState> =
        use_signal(ChannelConfigDialogState::default);
    let mut parameter_dialog_state: Signal<ParameterDialogState> =
        use_signal(ParameterDialogState::default);

    let mut render_nodes = use_signal(RequestRender::new);
    let mut render_channels = use_signal(RequestRender::new);

    use_context_provider(move || {
        let on_new_node = move |node_spec_key: NodeSpecKey| {
            //@todo improve ergonomics
            let mut service_ctx = use_context::<ServiceContext>();
            let mut write = service_ctx.service.write();
            let mut node_api_mut = write.node_api_mut();
            let mut lifecycle_api = node_api_mut.lifecycle();
            //@todo this should be returned as reference
            let specs_map = use_context::<Specs>();

            //@todo handle unwraps
            let node_spec = specs_map.nodes.spec(&node_spec_key).unwrap();
            let id = lifecycle_api.create(node_spec).unwrap();

            // make param state visible if params expected for this node spec
            if node_spec.params().is_some() {
                debug!("setting parameter dialog state");
                parameter_dialog_state.set(ParameterDialogState::Visible {
                    position: Point { x: 300.0, y: 200.0 },
                    id,
                });
            }

            //@todo improve default position
            let data = NodeUiData::default();
            let mut ui_api = write.ui_api_mut();
            ui_api.node().create(id, data).unwrap();
            info!(
                "created node {id} with name {} and {:?} kind",
                node_spec_key.name(),
                node_spec_key.kind()
            );
            render_nodes.with_mut(|write| write.request());
        };

        SidebarEventHandlers::new(on_new_node)
    });

    use_context_provider(move || {
        let on_new_channel = move |config: ChannelConfig| {
            if let ChannelConfigDialogState::Visible { spec, .. } =
                channel_config_dialog_state.take()
            {
                let mut service_ctx = use_context::<ServiceContext>();
                let mut write = service_ctx.service.write();
                let id = {
                    let mut channel_api = write.channel_api_mut();
                    channel_api.create(spec, config).unwrap()
                };
                {
                    //@todo improve default position
                    let data = ChannelUiData::default();
                    let mut ui_api = write.ui_api_mut();
                    ui_api.channel().create(id, data).unwrap();
                }
                render_channels.with_mut(|write| write.request());
            }
        };

        let on_cancel = move |_| {
            channel_config_dialog_state.take();
        };
        channel::config_dialog::Handlers::new(on_new_channel, on_cancel)
    });

    use_context_provider(move || parameter_dialog_handlers(parameter_dialog_state));

    rsx! {
        div { style: "display: flex; flex-direction: row; gap: 10px;",
            div { style: "flex: 0 1 20%;",
                Sidebar { channel_config_dialog_state }
            }
            div { style: "flex: 0 1 80%;",
                Workspace { render_nodes, render_channels }
            }
            div {
                ParameterDialog { state: parameter_dialog_state }
            }
            div { style: "flex: 0 1 10%;", Toolbar {} }
        }
    }
}

fn parameter_dialog_handlers(mut state: Signal<ParameterDialogState>) -> ParameterDialogHandlers {
    let on_confirm = move |(node_id, params): (NodeId, Parameters)| -> Result<()> {
        let mut service_ctx = use_context::<ServiceContext>();
        let mut write = service_ctx.service.write();
        let mut node_api = write.node_api_mut();
        let mut node_api_params = node_api.parameters();
        node_api_params.create(node_id, ParameterValue { params })?;
        state.take();
        Ok(())
    };

    let on_cancel = move |_| {
        state.take();
    };

    ParameterDialogHandlers::new(on_confirm, on_cancel)
}

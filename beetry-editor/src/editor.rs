use beetry_editor_types::{ChannelUiData, NodeSpecKey, NodeUiData};
use beetry_reconstruction_types::channel::ChannelConfig;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::ui::node::{ParameterDialogHandlers, ParameterDialogState};
use crate::workspace::Workspace2;
use crate::{
    EditorService,
    domain::repository::{
        ChannelRepositoryFacade, EdgeRepository, NodeRepositoryFacade, UiRepositoryFacade,
    },
};
use crate::{NodeSpecMap, toolbar::Toolbar2};
use crate::{Specs2, sidebar::Sidebar2};
use crate::{
    sidebar::SidebarEventHandlers2, ui::channel::config_dialog::State as ChannelConfigDialogState,
};
use crate::{
    signals::RequestRender,
    ui::channel::{self},
};

// fn parameter_dialog_handlers(
//     mut state: Signal<ParameterDialogState>,
//     sidebar_handlers: SidebarEventHandlers,
// ) -> ParameterDialogHandlers {
//     let on_confirm = move |(spec, params): (LeafSpec, Parameters)| {
//         sidebar_handlers.on_new_node.call(
//             ui::Node::new(
//                 spec.name,
//                 ui::NodeKind::Leaf {
//                     schema: spec.schema,
//                     external_receivers: Default::default(),
//                 },
//             )
//             .with_params(params),
//         );
//         state.take();
//     };

//     let on_cancel = move |_| {
//         state.take();
//     };

//     ParameterDialogHandlers::new(on_confirm, on_cancel)
// }

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
pub(crate) fn Editor2() -> Element {
    debug!("rendering editor");

    let specs = use_context::<Specs2>();
    use_context_provider(|| ServiceContext::new(specs.nodes));

    // let ctx = use_context_provider(EditorContext::new);
    // let channel_ctx = ctx.channel;
    let mut channel_config_dialog_state: Signal<ChannelConfigDialogState> =
        use_signal(ChannelConfigDialogState::default);
    let parameter_dialog_state: Signal<ParameterDialogState> =
        use_signal(ParameterDialogState::default);

    let mut render_nodes = use_signal(RequestRender::new);
    let mut render_channels = use_signal(RequestRender::new);

    let sidebar_handlers = use_context_provider(move || {
        let on_new_node = move |node_spec_key: NodeSpecKey| {
            //@todo improve ergonomics
            let mut service_ctx = use_context::<ServiceContext>();
            let mut write = service_ctx.service.write();
            let mut node_api_mut = write.node_api_mut();
            let mut lifecycle_api = node_api_mut.lifecycle();
            //@todo this should be returned as reference
            let specs_map = use_context::<Specs2>();

            //@todo handle unwraps
            let node_spec = specs_map.nodes.spec(&node_spec_key).unwrap();
            let id = lifecycle_api.create(node_spec.clone()).unwrap();

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

        SidebarEventHandlers2::new(on_new_node)
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
        channel::config_dialog::Handlers2::new(on_new_channel, on_cancel)
    });

    // use_context_provider(move || {
    //     parameter_dialog_handlers2(parameter_dialog_state, sidebar_handlers)
    // });

    rsx! {
        div { style: "display: flex; flex-direction: row; gap: 10px;",
            div { style: "flex: 0 1 20%;",
                Sidebar2 { channel_config_dialog_state }
            }
            div { style: "flex: 0 1 80%;",
                Workspace2 { render_nodes, render_channels }
            }
            div { style: "flex: 0 1 10%;", Toolbar2 {} }
        }
    }
}

// fn parameter_dialog_handlers2(
//     mut state: Signal<ParameterDialogState2>,
//     sidebar_handlers: SidebarEventHandlers2,
// ) -> ParameterDialogHandlers {
//     let on_confirm = move |spec: beetry_editor_types::NodeSpecKey| {
//         sidebar_handlers.on_new_node.call(spec);
//         state.take();
//     };

//     let on_cancel = move |_| {
//         state.take();
//     };

//     ParameterDialogHandlers::new(on_confirm, on_cancel)
// }

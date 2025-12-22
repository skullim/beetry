use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use beetry_editor_types::{NodeKind, NodeSpecKey};
use beetry_plugin_types::node::{LeafSpec, NodeName};
use beetry_reconstruction_types::{
    channel::{ChannelMetadata, ChannelSnapshot},
    parameter::Parameters,
};
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::ui::channel::{self};
use crate::ui::node::{ParameterDialogHandlers, ParameterDialogState};
use crate::ui::{self, edge};
use crate::workspace::{Workspace, Workspace2};
use crate::{
    EditorService,
    definitions::NodeId,
    domain::repository::{
        ChannelRepositoryFacade, EdgeRepository, NodeRepositoryFacade, UiRepositoryFacade,
    },
};
use crate::{
    NodeSpecMap,
    toolbar::{Toolbar, Toolbar2},
};
use crate::{
    Specs2,
    sidebar::{Sidebar, Sidebar2, SidebarEventHandlers},
};
use crate::{
    sidebar::SidebarEventHandlers2, ui::channel::config_dialog::State as ChannelConfigDialogState,
};

#[derive(Debug, Clone)]
struct EditorContext {
    id: Signal<NodeId>,
    edge: edge::Context,
    channel: channel::Context,
    ui_nodes: Signal<ui::NodeMap>,
}

impl EditorContext {
    fn new() -> Self {
        let mut id = Signal::new(0);
        let ui_nodes = Signal::new({
            let mut map = ui::NodeMap::new();
            map.insert(
                id(),
                ui::Node::new(NodeName::new("Root"), ui::NodeKind::Root),
            );
            id += 1;
            map
        });
        Self {
            id,
            edge: edge::Context::new(),
            channel: channel::Context::new(),
            ui_nodes,
        }
    }
}

#[derive(Default, Clone)]
pub struct NodeIdToNameStorage {
    pub map: HashMap<NodeId, NodeName>,
}

pub type SharedNodeIdToNameStorage = Rc<RefCell<NodeIdToNameStorage>>;

#[component]
pub(crate) fn Editor() -> Element {
    debug!("rendering editor");

    let ctx = use_context_provider(EditorContext::new);
    let mut id = ctx.id;
    let mut ui_nodes = ctx.ui_nodes;
    let edge_ctx = ctx.edge;
    let channel_ctx = ctx.channel;

    let channel_config_dialog_state: Signal<ChannelConfigDialogState> =
        use_signal(ChannelConfigDialogState::default);

    let parameter_dialog_state: Signal<ParameterDialogState> =
        use_signal(ParameterDialogState::default);

    use_context_provider(SharedNodeIdToNameStorage::default);

    let sidebar_handlers = use_context_provider(move || {
        let on_new_node = move |node: ui::Node| {
            let storage = use_context::<SharedNodeIdToNameStorage>();
            storage.borrow_mut().map.insert(id(), node.name.clone());
            ui_nodes.write().insert(id(), node);
            id += 1;
        };

        SidebarEventHandlers::new(on_new_node)
    });

    use_context_provider(move || {
        channel_config_dialog_handlers(channel_ctx.tracker, channel_config_dialog_state)
    });

    use_context_provider(move || {
        parameter_dialog_handlers(parameter_dialog_state, sidebar_handlers)
    });

    rsx! {
        div { style: "display: flex; flex-direction: row; gap: 10px;",
            div { style: "flex: 0 1 20%;",
                Sidebar {
                    channel_config_dialog_state,
                    parameter_dialog_state,
                }
            }
            div { style: "flex: 0 1 80%;",
                Workspace { ui_nodes, edge_ctx, channel_ctx }
            }
            div { style: "flex: 0 1 10%;",
                Toolbar {
                    id,
                    ui_nodes,
                    edge_tracker: edge_ctx.tracker,
                    channel_tracker: channel_ctx.tracker,
                }
            }
        }
    }
}

fn on_new_channel(snapshot: ChannelSnapshot, mut tracker: Signal<channel::Tracker>) {
    tracker.with_mut(|tracker| {
        tracker.create_channel(snapshot);
    });
}

fn channel_config_dialog_handlers(
    tracker: Signal<channel::Tracker>,
    mut state: Signal<channel::config_dialog::State>,
) -> channel::config_dialog::Handlers {
    let on_confirm = move |channel_metadata: ChannelMetadata| {
        if let ChannelConfigDialogState::Visible { position: _, spec } = state.take() {
            on_new_channel(ChannelSnapshot::new(spec, channel_metadata), tracker);
        }
    };

    let on_cancel = move |_| {
        state.take();
    };

    channel::config_dialog::Handlers::new(on_confirm, on_cancel)
}

fn parameter_dialog_handlers(
    mut state: Signal<ParameterDialogState>,
    sidebar_handlers: SidebarEventHandlers,
) -> ParameterDialogHandlers {
    let on_confirm = move |(spec, params): (LeafSpec, Parameters)| {
        sidebar_handlers.on_new_node.call(
            ui::Node::new(
                spec.name,
                ui::NodeKind::Leaf {
                    schema: spec.schema,
                    external_receivers: Default::default(),
                },
            )
            .with_params(params),
        );
        state.take();
    };

    let on_cancel = move |_| {
        state.take();
    };

    ParameterDialogHandlers::new(on_confirm, on_cancel)
}

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

    let ctx = use_context_provider(EditorContext::new);
    let channel_ctx = ctx.channel;
    let channel_config_dialog_state: Signal<ChannelConfigDialogState> =
        use_signal(ChannelConfigDialogState::default);
    let parameter_dialog_state: Signal<ParameterDialogState> =
        use_signal(ParameterDialogState::default);

    let sidebar_handlers = use_context_provider(move || {
        let on_new_node = |node_spec_key: NodeSpecKey| {
            let mut service_ctx = use_context::<ServiceContext>();

            let mut write = service_ctx.service.write();
            let mut node_api_mut = write.node_api_mut();
            let mut lifecycle_api = node_api_mut.lifecycle();
            //@todo this should be returned as reference
            let specs_map = use_context::<Specs2>();

            //@todo handle unwraps
            let node_spec = specs_map.nodes.spec(&node_spec_key).unwrap();
            lifecycle_api.create(node_spec.clone()).unwrap();
            //@todo missing instantiation of ui element, maybe pass position into closure?

            //@todo root node has to be created once at the initialization phase
        };

        SidebarEventHandlers2::new(on_new_node)
    });

    use_context_provider(move || {
        channel_config_dialog_handlers(channel_ctx.tracker, channel_config_dialog_state)
    });

    // use_context_provider(move || {
    //     parameter_dialog_handlers2(parameter_dialog_state, sidebar_handlers)
    // });

    rsx! {
        div { style: "display: flex; flex-direction: row; gap: 10px;",
            div { style: "flex: 0 1 20%;",
                Sidebar2 {
                    channel_config_dialog_state,
                    //parameter_dialog_state,
                }
            }
            div { style: "flex: 0 1 80%;", Workspace2 {} }
            div { style: "flex: 0 1 10%;", Toolbar2 {
            } }
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

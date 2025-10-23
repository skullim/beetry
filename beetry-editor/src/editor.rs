use std::collections::BTreeSet;

use beetry_serde::de::channel::{ChannelMetadata, ChannelSnapshot};
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::{
    definitions::NodeId,
    sidebar::{Sidebar, SidebarEventHandlers},
    toolbar::Toolbar,
    ui::{
        self,
        channel::{self, config_dialog::State as ChannelConfigDialogState},
        edge,
        node::{ParameterDialogHandlers, ParameterDialogState},
    },
    workspace::Workspace,
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
            map.insert(id(), ui::Node::new(ui::NodeKind::Root));
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

    let sidebar_handlers = use_context_provider(move || {
        let on_new_node = move |ui: ui::NodeKind| {
            ui_nodes.write().insert(id(), ui::Node::new(ui));
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
    let on_confirm = move |(spec, params)| {
        sidebar_handlers.on_new_node.call(ui::NodeKind::Leaf {
            spec,
            params,
            external_receivers: BTreeSet::new(),
        });
        state.take();
    };

    let on_cancel = move |_| {
        state.take();
    };

    ParameterDialogHandlers::new(on_confirm, on_cancel)
}

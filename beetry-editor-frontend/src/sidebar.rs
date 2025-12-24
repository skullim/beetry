use beetry_editor_types::NodeSpecKey;
use dioxus::logger::tracing::info;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::Specs2;
use crate::definitions::Point;
use crate::editor::ServiceContext;
use crate::ui::channel::config_dialog::State as ChannelConfigDialogState;
use crate::ui::channel::{self};

#[derive(Clone)]
pub struct SidebarEventHandlers2 {
    pub(crate) on_new_node: EventHandler<NodeSpecKey>,
}

impl SidebarEventHandlers2 {
    pub(crate) fn new(on_new_node: impl FnMut(NodeSpecKey) + 'static) -> Self {
        Self {
            on_new_node: EventHandler::new(on_new_node),
        }
    }
}

#[component]
pub(crate) fn Sidebar2(
    channel_config_dialog_state: Signal<ChannelConfigDialogState>,
    //parameter_dialog_state: Signal<node::ParameterDialogState>,
) -> Element {
    debug!("rendering sidebar");

    let on_new_node = use_context::<SidebarEventHandlers2>().on_new_node;
    use_hook(|| on_new_node.call(NodeSpecKey::root()));

    let specs = use_context::<Specs2>();
    let node_specs = specs.nodes;
    let channel_specs = specs.channels;

    info!("registered {} nodes", node_specs.values().count());

    let controls = node_specs
        .values()
        .filter(|v| v.kind() == beetry_editor_types::NodeKind::Control);

    let actions = node_specs
        .values()
        .filter(|v| v.kind() == beetry_editor_types::NodeKind::Action);
    let conditions = node_specs
        .values()
        .filter(|v| v.kind() == beetry_editor_types::NodeKind::Condition);

    let new_control_handler = |spec: NodeSpecKey| {
        move |_| {
            on_new_node.call(spec.clone());
        }
    };

    let new_leaf_handler = |spec_key: NodeSpecKey| {
        move |_| {
            on_new_node.call(spec_key.clone());
            let mut service_ctx = use_context::<ServiceContext>();
            let mut write = service_ctx.service.write();

            // let schema = spec_key.schema();
            // let params_schema = spec_key.params_schema();
            // let params_len = params_schema.defs.len();
            // if params_schema.defs.is_empty() {
            //     debug!(
            //         "no parameters needed for '{}', creating node directly",
            //         spec_key.name()
            //     );
            //     on_new_node.call(spec_key);
            // } else {
            //     debug!(
            //         "parameter dialog for '{}' with {params_len} parameters",
            //         spec_key.name(),
            //     );
            //     parameter_dialog_state.set(node::ParameterDialogState::Visible {
            //         position: Point { x: 300.0, y: 200.0 },
            //         spec: spec_key.clone(),
            //     });
            // }
        }
    };

    rsx! {
        div {
            h3 { "Control Nodes" }
            for spec in controls {
                button { onclick: new_control_handler(spec.key().clone()), {format!("{}", spec.name())} }
            }

            h3 { "Action Nodes" }
            for spec in actions {
                button { onclick: new_leaf_handler(spec.key().clone()), {format!("{}", spec.name())} }
            }

            h3 { "Condition Nodes" }
            for spec in conditions {
                button { onclick: new_leaf_handler(spec.key().clone()), {format!("{}", spec.name())} }
            }

            h3 { "Channels" }
            for spec in channel_specs.values().cloned() {
                button {
                    onclick: move |_| {
                        channel_config_dialog_state
                            .set(ChannelConfigDialogState::Visible {
                                position: Point { x: 200.0, y: 100.0 },
                                spec: spec.clone(),
                            });
                    },
                    {spec.as_str()}
                }
            }

            div {
                channel::ConfigDialog2 { state: channel_config_dialog_state }
            }
        //         div {
        //     node::ParameterDialog { state: parameter_dialog_state }
        // }
        }
    }
}

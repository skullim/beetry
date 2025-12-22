use std::collections::HashSet;

use beetry_editor_types::NodeSpecKey;
use beetry_plugin_types::node::{ControlSpec, LeafKind, LeafSpec};
use dioxus::logger::tracing::info;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::definitions::Point;
use crate::ui::channel::config_dialog::State as ChannelConfigDialogState;
use crate::ui::channel::{self};
use crate::ui::{self, node};
use crate::{SpecPlugins, Specs2};

#[derive(Clone)]
pub struct SidebarEventHandlers {
    pub(crate) on_new_node: EventHandler<ui::Node>,
}

impl SidebarEventHandlers {
    pub(crate) fn new(on_new_node: impl FnMut(ui::Node) + 'static) -> Self {
        Self {
            on_new_node: EventHandler::new(on_new_node),
        }
    }
}

#[component]
pub(crate) fn Sidebar(
    channel_config_dialog_state: Signal<ChannelConfigDialogState>,
    parameter_dialog_state: Signal<node::ParameterDialogState>,
) -> Element {
    let on_new_node = use_context::<SidebarEventHandlers>().on_new_node;

    let plugins = use_context::<SpecPlugins>();
    let leaves = plugins.leaves;
    let controls = plugins.controls;
    let channels = plugins.channels;

    debug!("rendering sidebar");
    info!("registered {} leaf nodes", leaves.len());

    let new_control_handler = |spec: ControlSpec| {
        move |_| {
            on_new_node.call(ui::Node::new(
                spec.name.clone(),
                ui::NodeKind::Control {
                    params_schema: spec.params_schema.clone(),
                },
            ));
        }
    };

    let actions = leaves
        .iter()
        .filter(|spec| matches!(spec.schema().kind(), LeafKind::Action))
        .cloned();

    let conditions = leaves
        .iter()
        .filter(|spec| matches!(spec.schema().kind(), LeafKind::Condition))
        .cloned();

    let new_leaf_handler = |spec: LeafSpec| {
        move |_| {
            let schema = spec.schema();
            let params_schema = spec.params_schema();
            let params_len = params_schema.defs.len();
            debug!(
                "creating node '{}' with {params_len} parameters",
                spec.name(),
            );
            if params_schema.defs.is_empty() {
                debug!(
                    "no parameters needed for '{}', creating node directly",
                    spec.name()
                );
                on_new_node.call(ui::Node::new(
                    spec.name.clone(),
                    ui::NodeKind::Leaf {
                        schema: schema.clone(),
                        external_receivers: HashSet::new(),
                    },
                ));
            } else {
                debug!(
                    "parameter dialog for '{}' with {params_len} parameters",
                    spec.name(),
                );
                parameter_dialog_state.set(node::ParameterDialogState::Visible {
                    position: Point { x: 300.0, y: 200.0 },
                    spec: spec.clone(),
                });
            }
        }
    };

    rsx! {
        div {
            h3 { "Control Nodes" }
            for spec in controls {
                button { onclick: new_control_handler(spec), {format!("{}", spec.name())} }
            }

            h3 { "Action Nodes" }
            for spec in actions {
                button { onclick: new_leaf_handler(spec), {format!("{}", spec.name())} }
            }

            h3 { "Condition Nodes" }
            for spec in conditions {
                button { onclick: new_leaf_handler(spec), {format!("{}", spec.name())} }
            }

            h3 { "Channels" }
            for spec in channels {
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
                channel::ConfigDialog { state: channel_config_dialog_state }
            }

            div {
                node::ParameterDialog { state: parameter_dialog_state }
            }
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
            // let schema = spec_key.schema();
            // let params_schema = spec_key.params_schema();
            // let params_len = params_schema.defs.len();
            // debug!(
            //     "creating node '{}' with {params_len} parameters",
            //     spec_key.name(),
            // );
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
                channel::ConfigDialog { state: channel_config_dialog_state }
            }

            // div {
            //     node::ParameterDialog { state: parameter_dialog_state }
            // }
        }
    }
}

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

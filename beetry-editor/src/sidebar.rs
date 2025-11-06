use std::collections::BTreeSet;

use beetry_serde::ser::node::{ControlNodeSpec, LeafKind, LeafNodeSpec};
use dioxus::{logger::tracing::info, prelude::*};
use dioxus_logger::tracing::debug;

use crate::{
    Plugins,
    definitions::Point,
    ui::{
        self,
        channel::{self, config_dialog::State as ChannelConfigDialogState},
        node,
    },
};

#[derive(Clone)]
pub(crate) struct SidebarEventHandlers {
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

    let plugins = use_context::<Plugins>();
    let leaves = &plugins.leaves;
    let controls = plugins.controls;
    let channels = plugins.channels;

    debug!("rendering sidebar");
    info!("registered {} leaf nodes", leaves.len());

    let new_control_handler = |spec: ControlNodeSpec| {
        move |_| {
            on_new_node.call(ui::Node::new(
                spec.name.clone(),
                ui::NodeKind::Control {
                    params_schema: spec.params_schema.clone(),
                },
            ));
        }
    };

    let actions: Vec<_> = leaves
        .iter()
        .filter(|spec| matches!(spec.schema().kind(), LeafKind::Action))
        .cloned()
        .collect();

    let conditions: Vec<_> = leaves
        .iter()
        .filter(|spec| matches!(spec.schema().kind(), LeafKind::Condition))
        .cloned()
        .collect();

    let new_leaf_handler = |spec: LeafNodeSpec| {
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
                        external_receivers: BTreeSet::new(),
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

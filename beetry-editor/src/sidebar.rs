use std::collections::BTreeSet;

use beetry_serde::{
    de::{node::ControlKind, parameter::Parameters},
    ser::node::{LeafKind, LeafSpec},
};
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
    pub(crate) on_new_node: EventHandler<ui::NodeKind>,
}

impl SidebarEventHandlers {
    pub(crate) fn new(on_new_node: impl FnMut(ui::NodeKind) + 'static) -> Self {
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
    let channels = plugins.channels;

    debug!("rendering sidebar");
    info!("registered {} leaf nodes", leaves.len());

    let controls = [
        ("Sequence", ControlKind::Sequence),
        ("Fallback", ControlKind::Fallback),
        ("Parallel", ControlKind::Parallel),
    ];
    let new_control_handler = |kind: ControlKind| {
        move |_| {
            on_new_node.call(ui::NodeKind::Control(kind));
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

    let new_leaf_handler = |spec: LeafSpec| {
        move |_| {
            let schema = spec.schema().params().clone();
            debug!(
                "creating node '{}' with {} parameters",
                spec.name(),
                schema.params.len()
            );
            if schema.params.is_empty() {
                debug!(
                    "no parameters needed for '{}', creating node directly",
                    spec.name()
                );
                on_new_node.call(ui::NodeKind::Leaf {
                    spec: spec.clone(),
                    params: Parameters::default(),
                    external_receivers: BTreeSet::new(),
                });
            } else {
                debug!(
                    "parameter dialog for '{}' with {} parameters",
                    spec.name(),
                    schema.params.len()
                );
                parameter_dialog_state.set(node::ParameterDialogState::Visible {
                    position: Point { x: 300.0, y: 200.0 },
                    spec: spec.clone(),
                    schema,
                });
            }
        }
    };

    rsx! {
        div {
            h3 { "Control Nodes" }
            for (label , kind) in controls {
                button { onclick: new_control_handler(kind), {label} }
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

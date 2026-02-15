use crate::ui::handler::handlers;
use crate::{Point, SharedSpecs};
use beetry_editor_types::spec::node::{NodeKind, NodeSpecKey};
use dioxus::logger::tracing::info;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::ui::channel::config_dialog::State as ChannelConfigDialogState;
use crate::ui::channel::{self};

handlers!(on_new_node: NodeSpecKey);

#[component]
pub(crate) fn Sidebar(channel_config_dialog_state: Signal<ChannelConfigDialogState>) -> Element {
    debug!("rendering");

    let on_new_node = use_context::<Handlers>().on_new_node;
    use_hook(|| on_new_node.call(NodeSpecKey::root()));

    let specs = use_context::<SharedSpecs>();
    let node_specs = &specs.nodes;
    let channel_specs = &specs.channels;

    info!("registered {} nodes", node_specs.values().count());

    let controls = node_specs
        .values()
        .filter(|v| v.kind() == NodeKind::Control);

    let actions = node_specs
        .values()
        .filter(|v| v.kind() == NodeKind::action());
    let conditions = node_specs
        .values()
        .filter(|v| v.kind() == NodeKind::condition());

    let new_node_handler = |spec: NodeSpecKey| {
        move |_| {
            on_new_node.call(spec.clone());
        }
    };

    rsx! {
        div {
            h3 { "Control Nodes" }
            for spec in controls {
                button { onclick: new_node_handler(spec.key().clone()), {format!("{}", spec.name())} }
            }

            h3 { "Action Nodes" }
            for spec in actions {
                button { onclick: new_node_handler(spec.key().clone()), {format!("{}", spec.name())} }
            }

            h3 { "Condition Nodes" }
            for spec in conditions {
                button { onclick: new_node_handler(spec.key().clone()), {format!("{}", spec.name())} }
            }

            h3 { "Channels" }
            for spec in channel_specs.values().cloned() {
                button {
                    onclick: move |_| {
                        channel_config_dialog_state
                            .set(ChannelConfigDialogState::Visible {
                                position: Point { x: 200.0, y: 100.0 },
                                spec_key: spec.msg_hash(),
                            });
                    },
                    {spec.as_str()}
                }
            }

            div {
                channel::ConfigDialog { state: channel_config_dialog_state }
            }
        }
    }
}

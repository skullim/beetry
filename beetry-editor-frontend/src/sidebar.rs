use crate::SharedSpecs;
use crate::ui::handler::define_handlers;
use beetry_editor_types::spec::channel::ChannelSpec;
use beetry_editor_types::spec::node::{NodeKind, NodeSpecKey};
use dioxus::logger::tracing::info;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

define_handlers!(on_new_node: NodeSpecKey,
                 on_new_channel: ChannelSpec);

#[component]
pub(crate) fn Sidebar() -> Element {
    debug!("rendering");
    let handlers = use_context::<Handlers>();

    use_hook(|| handlers.on_new_node.call(NodeSpecKey::root()));

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

    let on_new_node = |spec: NodeSpecKey| {
        move |_| {
            handlers.on_new_node.call(spec.clone());
        }
    };

    rsx! {
        div {
            h3 { "Control Nodes" }
            for spec in controls {
                button { onclick: on_new_node(spec.key().clone()), {format!("{}", spec.name())} }
            }

            h3 { "Action Nodes" }
            for spec in actions {
                button { onclick: on_new_node(spec.key().clone()), {format!("{}", spec.name())} }
            }

            h3 { "Condition Nodes" }
            for spec in conditions {
                button { onclick: on_new_node(spec.key().clone()), {format!("{}", spec.name())} }
            }

            h3 { "Channels" }
            for spec in channel_specs.values().cloned() {
                button { onclick: move |_| { handlers.on_new_channel.call(spec.clone()) },
                    {spec.as_str()}
                }
            }


        }
    }
}

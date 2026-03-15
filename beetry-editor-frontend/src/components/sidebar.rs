mod context;
mod handlers;
mod search;
mod section;

use beetry_editor_types::spec::{
    channel::ChannelSpec,
    node::{NodeKind, NodeSpecKey},
};
use dioxus::prelude::*;
use search::Search;
use section::{ChannelSection, NodeSection};

use crate::{SharedSpecs, components::editor, ui::handler::define_handlers};

define_handlers!(on_new_node: NodeSpecKey,
                 on_new_channel: ChannelSpec);

#[component]
pub(crate) fn Sidebar(editor_state: editor::State) -> Element {
    rsx! {
        context::Provider { editor_state, Layout {} }
    }
}

#[component]
fn Layout() -> Element {
    debug!("rendering");
    let handlers = use_context::<Handlers>();
    let search_query = use_signal(String::new);

    use_hook(|| handlers.on_new_node.call(NodeSpecKey::root()));

    let specs = use_context::<SharedSpecs>();
    let node_specs = &specs.nodes;
    let channel_specs = &specs.channels;

    use_hook(move || {
        info!("registered {} nodes", node_specs.values().count());
        info!("registered {} channels", channel_specs.values().count());
    });

    let normalized_query = use_memo(move || search_query().trim().to_lowercase())();
    let matches_query = |label: &str| {
        normalized_query.is_empty() || label.to_lowercase().contains(&normalized_query)
    };

    let node_items = |kind: NodeKind| {
        node_specs
            .values()
            .filter(|v| v.kind() == kind)
            .map(|spec| (spec.key().clone(), spec.name().to_string()))
            .filter(|(_, label)| matches_query(label))
            .collect::<Vec<_>>()
    };
    let channel_items = || {
        channel_specs
            .values()
            .map(|spec| (spec.clone(), spec.as_str().to_string()))
            .filter(|(_, label)| matches_query(label))
            .collect::<Vec<_>>()
    };

    let controls = node_items(NodeKind::Control);
    let decorators = node_items(NodeKind::Decorator);
    let actions = node_items(NodeKind::action());
    let conditions = node_items(NodeKind::condition());
    let channels = channel_items();

    rsx! {
        div { class: "bt-sidebar",
            p { class: "bt-panel-title", "Sidebar" }
            Search { query: search_query }

            NodeSection { title: "Control Nodes", items: controls }
            NodeSection { title: "Decorator Nodes", items: decorators }
            NodeSection { title: "Action Nodes", items: actions }
            NodeSection { title: "Condition Nodes", items: conditions }
            ChannelSection { title: "Channels", items: channels }
        }
    }
}

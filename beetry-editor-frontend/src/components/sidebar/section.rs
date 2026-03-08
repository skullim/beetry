use super::Handlers;
use beetry_editor_types::spec::channel::ChannelSpec;
use beetry_editor_types::spec::node::NodeSpecKey;
use dioxus::prelude::*;

pub type NodeItems = Vec<(NodeSpecKey, String)>;
pub type ChannelItems = Vec<(ChannelSpec, String)>;

#[derive(Props, Clone, PartialEq)]
pub struct NodeSectionProps {
    pub title: &'static str,
    pub items: NodeItems,
}

#[component]
pub(crate) fn NodeSection(props: NodeSectionProps) -> Element {
    let handlers = use_context::<Handlers>();

    rsx! {
        section { class: "bt-sidebar-section",
            h3 { "{props.title}" }
            div { class: "bt-sidebar-list",
                for (spec_key , label) in props.items.iter().cloned() {
                    button {
                        class: "bt-btn bt-btn--sidebar",
                        onclick: move |_| handlers.on_new_node.call(spec_key.clone()),
                        "{label}"
                    }
                }
                if props.items.is_empty() {
                    p { class: "bt-sidebar-empty", "No matches" }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ChannelSectionProps {
    pub title: &'static str,
    pub items: ChannelItems,
}

#[component]
pub(crate) fn ChannelSection(props: ChannelSectionProps) -> Element {
    let handlers = use_context::<Handlers>();

    rsx! {
        section { class: "bt-sidebar-section",
            h3 { "{props.title}" }
            div { class: "bt-sidebar-list",
                for (spec , label) in props.items.iter().cloned() {
                    button {
                        class: "bt-btn bt-btn--sidebar",
                        onclick: move |_| handlers.on_new_channel.call(spec.clone()),
                        "{label}"
                    }
                }
                if props.items.is_empty() {
                    p { class: "bt-sidebar-empty", "No matches" }
                }
            }
        }
    }
}

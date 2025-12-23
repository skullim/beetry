use std::collections::HashSet;
use std::rc::Rc;

use beetry_core::MessageHash;
use beetry_plugin_types::node::{LeafKind, LeafSchema};
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::definitions::{NodeId, Point};
use crate::editor::{ServiceContext, SharedNodeIdToNameStorage};
use crate::ui::node::base::{NodeBase, NodeStyle, NodeWithContextMenu};
use crate::ui::node::port::{self, input};

#[derive(PartialEq, Clone, Props)]
pub struct LeafProps {
    id: NodeId,
    position: Point,
    //@todo would be better if this is accessible as lookup based on NodeId
    schema: LeafSchema,
    //@todo would be better if this is accessible as lookup based on NodeId
    name: String,
    //@todo higher level can provide the info what channel type it is
    external_receivers: HashSet<MessageHash>,
}

#[component]
pub(crate) fn Leaf(props: LeafProps) -> Element {
    debug!("rendering leaf component: {}", props.id);
    let leaf_schema = props.schema;
    let id = props.id;
    let storage = use_context::<SharedNodeIdToNameStorage>();
    let storage_borrow = storage.borrow();
    let name = storage_borrow.map.get(&id).unwrap();

    let style = use_hook(|| Rc::new(style(leaf_schema.kind, &name.0)));
    let position = props.position;
    let external_receivers = props.external_receivers;

    let half_width = style.width / 2.0;
    let width = style.width;

    rsx! {
        g {
            NodeWithContextMenu {
                children: rsx! {
                    NodeBase { id, position, style }
                },
                id,
            }
        }
        g { transform: "translate({half_width}, 0)",
            input::Port { id, position }
        }

        g { transform: "translate(-80, 10)",
            for (idx , msg_spec) in leaf_schema.receivers.iter().enumerate() {
                port::Receiver {
                    key: "{idx}",
                    id,
                    position,
                    spec: msg_spec.clone(),
                    channel_idx: idx,
                    is_external: external_receivers.contains(&msg_spec.hash()),
                }
            }
        }

        g { transform: "translate({width}, 10)",
            for (idx , msg_spec) in leaf_schema.senders.iter().enumerate() {
                port::Sender {
                    key: "{idx}",
                    id,
                    position,
                    spec: msg_spec.clone(),
                    channel_idx: idx,
                }
            }
        }
    }
}

fn style(kind: LeafKind, name: &str) -> NodeStyle {
    let (fill_color, hover_color) = match kind {
        LeafKind::Action => ("url(#action-gradient)", "url(#action-hover)"),
        LeafKind::Condition => ("url(#condition-gradient)", "url(#condition-hover)"),
    };
    NodeStyle::builder()
        .fill_gradient(fill_color)
        .hover_gradient(hover_color)
        .label(name)
        .build()
}

pub(super) fn style_defs() -> Element {
    rsx! {
        defs {
            linearGradient { id: "action-gradient",
                stop { offset: "5%", stop_color: "#F59E0B" }
                stop { offset: "95%", stop_color: "#D97706" }
            }
            linearGradient { id: "condition-gradient",
                stop { offset: "0%", stop_color: "#6B7280" }
                stop { offset: "100%", stop_color: "#374151" }
            }

            linearGradient { id: "action-hover",
                stop { offset: "0%", stop_color: "#FBBF24" }
                stop { offset: "100%", stop_color: "#F59E0B" }
            }
            linearGradient { id: "condition-hover",
                stop { offset: "0%", stop_color: "#9CA3AF" }
                stop { offset: "100%", stop_color: "#6B7280" }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct LeafProps2 {
    id: NodeId,
    position: Point,
}

#[component]
pub(crate) fn Leaf2(props: LeafProps2) -> Element {
    let id = props.id;
    debug!("rendering leaf component: {id}");
    let service = use_context::<ServiceContext>();
    let read = service.service.read();
    let node_api = read.node_api();
    let spec_api = node_api.spec();
    let name = spec_api.name(id).unwrap();
    let kind = if spec_api.kind(id).unwrap() == beetry_editor_types::NodeKind::Action {
        LeafKind::Action
    } else {
        LeafKind::Condition
    };
    let ports_spec = spec_api.ports(id).unwrap();

    let style = use_hook(|| Rc::new(style(kind, &name.0)));
    let position = props.position;

    let half_width = style.width / 2.0;
    let width = style.width;

    rsx! {
        g {
            NodeWithContextMenu {
                children: rsx! {
                    NodeBase { id, position, style }
                },
                id,
            }
        }
        g { transform: "translate({half_width}, 0)",
            input::Port { id, position }
        }

        g { transform: "translate(-80, 10)",
            for (port_id , port_spec) in ports_spec.receivers() {
                port::Receiver2 {
                    key: "{port_id}",
                    id,
                    position,
                    msg_spec: port_spec.msg_spec.clone(),
                    port_id: *port_id,
                    //@todo provide from connection state store
                    is_external: false,
                }
            }
        }

        g { transform: "translate({width}, 10)",
            for (port_id , port_spec) in ports_spec.senders() {
                port::Sender2 {
                    key: "{port_id}",
                    id,
                    position,
                    msg_spec: port_spec.msg_spec.clone(),
                    port_id: *port_id,
                }
            }
        }
    }
}

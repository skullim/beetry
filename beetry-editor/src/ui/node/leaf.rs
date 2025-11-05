use std::{collections::BTreeSet, rc::Rc};

use beetry_core::MessageHash;
use beetry_serde::ser::node::{LeafKind, LeafSchema};
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::{
    definitions::{NodeId, Point},
    ui::node::{
        base::{NodeBase, NodeStyle, NodeWithContextMenu},
        port::{self, input},
    },
};

#[derive(PartialEq, Clone, Props)]
pub(crate) struct LeafProps {
    id: NodeId,
    position: Point,
    schema: LeafSchema,
    name: String,
    //@todo higher level can provide the info what channel type it is
    external_receivers: BTreeSet<MessageHash>,
}

#[component]
pub(crate) fn Leaf(props: LeafProps) -> Element {
    debug!("rendering leaf component: {}", props.id);
    let leaf_schema = props.schema;

    let style = use_hook(|| Rc::new(style(leaf_schema.kind, &props.name)));
    let position = props.position;
    let id = props.id;
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
                    is_external: external_receivers.contains(msg_spec.hash()),
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

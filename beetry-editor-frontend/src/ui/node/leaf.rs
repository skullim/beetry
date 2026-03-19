use std::rc::Rc;

use beetry_editor_backend::{api, api::SpecByNodeIdQuery};
use beetry_editor_types::{id::NodeId, spec::node::LeafKind};
use dioxus::prelude::*;

use crate::{
    Backend, Point,
    ui::{
        error::ErrorQueueState,
        node::{
            base::{NODE_WIDTH, NodeBase, NodeStyle, NodeWithMenu},
            pin::input,
            port,
        },
        style::text::{FONT_SIZE_NORMAL, truncate_label},
    },
};

fn style(kind: LeafKind, name: &str) -> NodeStyle {
    let label = truncate_label(name, FONT_SIZE_NORMAL, NODE_WIDTH);
    let (fill_color, hover_color) = match kind {
        LeafKind::Action => ("url(#action-gradient)", "url(#action-hover)"),
        LeafKind::Condition => ("url(#condition-gradient)", "url(#condition-hover)"),
    };
    NodeStyle::builder()
        .fill_gradient(fill_color)
        .hover_gradient(hover_color)
        .label(label)
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
pub struct LeafProps {
    id: NodeId,
    position: Point,
}

#[component]
pub(crate) fn Leaf(props: LeafProps) -> Element {
    let id = props.id;
    debug!("rendering (node id: {id})");
    let backend = use_context::<Backend>();
    let read = backend.read();
    let spec_query = api::node::spec::by_node_id(&(*read));

    let mut errors = use_context::<ErrorQueueState>();
    let Some((name, kind)) = (|| -> anyhow::Result<_> {
        let name = spec_query.name(id)?;
        let kind = spec_query
            .kind(id)?
            .leaf()
            .context("node kind is not leaf")?;
        Ok((name, kind))
    })()
    .map_err(|e| errors.push(e))
    .ok() else {
        return rsx! {};
    };

    let style = use_hook(|| Rc::new(style(kind, &name.0)));

    let half_width = style.width / 2.0;
    let dimensions = port::NodeDimensions {
        width: style.width,
        height: style.height,
    };

    let position = props.position;
    rsx! {
        g {
            NodeWithMenu {
                children: rsx! {
                    NodeBase { id, position, style }
                },
                id,
            }
        }
        g { transform: "translate({half_width}, 0)",
            input::Pin { id, position }
        }

        port::Renderer { id, position, dimensions }
    }
}

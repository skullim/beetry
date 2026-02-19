use crate::Backend;
use crate::Point;
use crate::ui::error::ErrorQueueState;
use crate::ui::node::base::{NodeBase, NodeStyle, NodeWithMenu};
use crate::ui::node::port::{self, input, layout as port_layout};
use beetry_editor_backend::api::SpecByNodeIdQueryView;
use beetry_editor_types::id::NodeId;
use beetry_editor_types::id::NodePortId;
use beetry_editor_types::spec::node::LeafKind;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;
use std::rc::Rc;

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
pub struct LeafProps {
    id: NodeId,
    position: Point,
}

struct PortIdRowIdxPair {
    port_id: NodePortId,
    row_idx: usize,
}

#[component]
pub(crate) fn Leaf(props: LeafProps) -> Element {
    let id = props.id;
    debug!("rendering (node id: {id})");
    let backend = use_context::<Backend>();
    let read = backend.read();
    let spec_query = beetry_editor_backend::api::node::spec::by_node_id(&(*read));

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
    let width = style.width;
    let height = style.height;
    let step_for = |count: usize| {
        let intervals = count.saturating_sub(1);
        if intervals == 0 {
            0.0
        } else {
            ((height - port_layout::HEIGHT) / intervals as f64)
                .max(port_layout::HEIGHT + port_layout::MIN_GAP)
        }
    };
    let receiver_step = use_hook(|| {
        step_for(
            spec_query
                .ports(id)
                .iter()
                .flat_map(|ports_spec| ports_spec.receiver_ids())
                .count(),
        )
    });
    let sender_step = use_hook(|| {
        step_for(
            spec_query
                .ports(id)
                .iter()
                .flat_map(|ports_spec| ports_spec.sender_ids())
                .count(),
        )
    });

    let receiver_id_pairs: Rc<Vec<_>> = use_hook(|| {
        Rc::new(
            spec_query
                .ports(id)
                .unwrap()
                .receiver_ids()
                .enumerate()
                .map(|(row_idx, port_id)| PortIdRowIdxPair {
                    port_id: *port_id,
                    row_idx,
                })
                .collect(),
        )
    });

    let sender_id_pairs: Rc<Vec<_>> = use_hook(|| {
        Rc::new(
            spec_query
                .ports(id)
                .unwrap()
                .sender_ids()
                .enumerate()
                .map(|(row_idx, port_id)| PortIdRowIdxPair {
                    port_id: *port_id,
                    row_idx,
                })
                .collect(),
        )
    });

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
            input::Port { id, position }
        }

        g {
            for PortIdRowIdxPair { row_idx , port_id } in receiver_id_pairs.iter() {
                port::Receiver {
                    key: "{*port_id}",
                    id,
                    position: Point {
                        x: position.x,
                        y: position.y + *row_idx as f64 * receiver_step,
                    },
                    port_id: *port_id,
                }
            }
        }

        g {
            for PortIdRowIdxPair { row_idx , port_id } in sender_id_pairs.iter() {
                port::Sender {
                    key: "{*port_id}",
                    id,
                    position: Point {
                        x: position.x + width,
                        y: position.y + *row_idx as f64 * sender_step,
                    },
                    port_id: *port_id,
                }
            }
        }
    }
}

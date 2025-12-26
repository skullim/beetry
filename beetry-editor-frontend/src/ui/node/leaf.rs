use crate::Point;
use crate::editor::ServiceContext;
use crate::ui::node::base::{NodeBase, NodeStyle, NodeWithContextMenu};
use crate::ui::node::port::{self, input};
use beetry_editor_types::id::NodeId;
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

#[component]
pub(crate) fn Leaf(props: LeafProps) -> Element {
    let id = props.id;
    debug!("rendering leaf component: {id}");
    let service = use_context::<ServiceContext>();
    let read = service.service.read();
    let node_api = read.node_api();
    let spec_api = node_api.spec();
    let name = spec_api.name(id).unwrap();
    let kind = spec_api.kind(id).unwrap().leaf().unwrap();
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
                port::Receiver {
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
                port::Sender {
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

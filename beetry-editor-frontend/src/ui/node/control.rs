use crate::Backend;
use crate::Point;
use crate::ui::error::ErrorQueueState;
use crate::ui::node::base::{NodeBase, NodeStyle, NodeWithMenu};
use crate::ui::node::pin::{input, output};
use beetry_editor_backend::api;
use beetry_editor_backend::api::SpecByNodeIdQuery;
use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;
use std::rc::Rc;

fn style(name: &str) -> NodeStyle {
    NodeStyle::builder()
        .fill_gradient("url(#control-gradient)")
        .hover_gradient("url(#control-hover)")
        .label(name)
        .height(70.0)
        .build()
}

pub(super) fn style_defs() -> Element {
    rsx! {
        defs {
            linearGradient { id: "control-gradient",
                stop { offset: "0%", stop_color: "#14B8A6" }
                stop { offset: "100%", stop_color: "#0F766E" }
            }
            linearGradient { id: "control-hover",
                stop { offset: "0%", stop_color: "#5EEAD4" }
                stop { offset: "100%", stop_color: "#14B8A6" }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ControlProps {
    id: NodeId,
    position: Point,
}

#[component]
pub fn Control(props: ControlProps) -> Element {
    let id = props.id;
    debug!("rendering (node id: {id})");
    let backend = use_context::<Backend>();
    let read = backend.read();
    let spec_query = api::node::spec::by_node_id(&(*read));

    let mut errors = use_context::<ErrorQueueState>();
    let Some(name) = spec_query.name(id).map_err(|e| errors.push(e)).ok() else {
        return rsx!();
    };

    let style = use_hook(|| Rc::new(style(&name.0)));
    let position = props.position;
    let half_width = style.width / 2.0;
    let height = style.height;

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
        g { transform: "translate({half_width}, {height})",
            output::Pin { id, position }
        }
    }
}

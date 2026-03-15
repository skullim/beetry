use std::rc::Rc;

use beetry_editor_backend::{api, api::SpecByNodeIdQuery};
use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;

use crate::{
    Backend, Point,
    ui::{
        error::ErrorQueueState,
        node::{
            base::{NodeBase, NodeStyle, NodeWithMenu},
            pin::{input, output},
        },
    },
};

fn style(name: &str) -> NodeStyle {
    NodeStyle::builder()
        .fill_gradient("url(#decorator-gradient)")
        .hover_gradient("url(#decorator-hover)")
        .label(name)
        .height(70.0)
        .build()
}

pub(super) fn style_defs() -> Element {
    rsx! {
        defs {
            linearGradient { id: "decorator-gradient",
                stop { offset: "0%", stop_color: "#0EA5E9" }
                stop { offset: "100%", stop_color: "#0369A1" }
            }
            linearGradient { id: "decorator-hover",
                stop { offset: "0%", stop_color: "#38BDF8" }
                stop { offset: "100%", stop_color: "#0EA5E9" }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct DecoratorProps {
    id: NodeId,
    position: Point,
}

#[component]
pub fn Decorator(props: DecoratorProps) -> Element {
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

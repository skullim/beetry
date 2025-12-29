use std::rc::Rc;

use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::Point;
use crate::ui::node::base::{NodeBase, NodeStyle};
use crate::ui::node::port::output;

#[derive(Props, PartialEq, Clone)]
pub struct RootProps {
    id: NodeId,
    position: Point,
}

#[component]
pub(crate) fn Root(props: RootProps) -> Element {
    debug!("rendering root: {}", props.id);

    let style = use_hook(|| Rc::new(style()));
    let id = props.id;
    let position = props.position;
    let half_width = style.width / 2.0;
    let height = style.height;

    rsx! {
        g {
            NodeBase { id, position, style }
        }
        g { transform: "translate({half_width}, {height})",
            output::Port { id, position }
        }
    }
}

fn style() -> NodeStyle {
    NodeStyle::builder()
        .fill_gradient("url(#root-gradient)")
        .hover_gradient("url(#root-hover)")
        .label("Root")
        .height(70.0)
        .build()
}

pub(super) fn style_defs() -> Element {
    rsx! {
        defs {
            linearGradient { id: "root-gradient",
                stop { offset: "5%", stop_color: "#4F46E5" }
                stop { offset: "95%", stop_color: "#3730A3" }
            }
            linearGradient { id: "root-hover",
                stop { offset: "0%", stop_color: "#6366F1" }
                stop { offset: "100%", stop_color: "#4F46E5" }
            }
        }
    }
}

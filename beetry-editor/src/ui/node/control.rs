use std::rc::Rc;

use beetry_editor_types::NodeId;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::definitions::Point;
use crate::editor::ServiceContext;
use crate::ui::node::base::{NodeBase, NodeStyle, NodeWithContextMenu};
use crate::ui::node::port::{input, output};

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
pub struct ControlProps2 {
    id: NodeId,
    position: Point,
}

#[component]
pub fn Control2(props: ControlProps2) -> Element {
    let id = props.id;
    debug!("rendering control component: {id}");
    let service = use_context::<ServiceContext>();
    let read = service.service.read();
    let node_api = read.node_api();
    let spec_api = node_api.spec();
    let name = spec_api.name(id).unwrap();

    let style = use_hook(|| Rc::new(style(&name.0)));
    let position = props.position;
    let half_width = style.width / 2.0;
    let height = style.height;

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
        g { transform: "translate({half_width}, {height})",
            output::Port { id, position }
        }
    }
}

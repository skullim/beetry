use std::rc::Rc;

use beetry_definitions::export::ControlKind;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::{
    definitions::{NodeId, Point},
    ui::node::{
        base::{NodeBase, NodeStyle, NodeWithContextMenu},
        port::{input, output},
    },
};

#[derive(Props, PartialEq, Clone)]
pub(crate) struct ControlProps {
    id: NodeId,
    position: Point,
    kind: ControlKind,
}

#[component]
pub fn Control(props: ControlProps) -> Element {
    debug!("rendering control component: {}", props.id);

    let style = use_hook(|| Rc::new(style(props.kind)));
    let id = props.id;
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

fn style(kind: ControlKind) -> NodeStyle {
    let label = match kind {
        ControlKind::Sequence => "Sequence",
        ControlKind::Fallback => "Fallback",
        ControlKind::Parallel => "Parallel",
    };

    NodeStyle::builder()
        .fill_gradient("url(#control-gradient)")
        .hover_gradient("url(#control-hover)")
        .label(label)
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

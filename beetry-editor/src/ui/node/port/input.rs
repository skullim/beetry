use dioxus::prelude::*;

use crate::definitions::{NodeId, Point};

#[derive(Debug, Clone)]
pub struct Handlers {
    pub(crate) on_mouse_up: EventHandler<NodeId>,
}

impl Handlers {
    pub(crate) fn new(on_mouse_up: impl FnMut(NodeId) + 'static) -> Self {
        Self {
            on_mouse_up: EventHandler::new(on_mouse_up),
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct PortProps {
    id: NodeId,
    position: Point,
}

#[component]
pub(crate) fn Port(props: PortProps) -> Element {
    let position = props.position;

    let mut is_hovered = use_signal(|| false);
    let fill_gradient = if *is_hovered.read() {
        "url(#io-port-hover)"
    } else {
        "url(#io-port-gradient)"
    };
    let port_radius = 7.0;

    rsx! {
        g {
            circle {
                cx: "{position.x}",
                cy: "{position.y - port_radius}",
                r: "{port_radius}",
                fill: "{fill_gradient}",
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1.5",
                onmouseenter: move |_| is_hovered.set(true),
                onmouseleave: move |_| is_hovered.set(false),
                onmouseup: move |_| { use_context::<Handlers>().on_mouse_up.call(props.id) },
            }

            circle {
                cx: "{position.x}",
                cy: "{position.y - port_radius}",
                r: "2.5",
                fill: "rgba(255,255,255,0.4)",
                pointer_events: "none",
            }
        }
    }
}

pub(super) fn style_defs() -> Element {
    rsx! {
        defs {
            linearGradient { id: "io-port-gradient",
                stop { offset: "0%", stop_color: "#8B5CF6" }
                stop { offset: "100%", stop_color: "#7C3AED" }
            }

            linearGradient { id: "io-port-hover",
                stop { offset: "0%", stop_color: "#A78BFA" }
                stop { offset: "100%", stop_color: "#8B5CF6" }
            }
        }
    }
}

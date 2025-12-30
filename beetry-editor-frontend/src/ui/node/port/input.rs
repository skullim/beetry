use crate::{Point, ui::node::port::IoPortStyleUrl};
use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;

#[derive(Debug, Clone)]
pub struct Handlers {
    pub(crate) on_mouse_up: EventHandler<NodeId>,
}

impl Handlers {
    pub(crate) fn new(on_mouse_up: impl FnMut(NodeId) -> Result<()> + 'static) -> Self {
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
    static PORT_RADIUS: f64 = 7.0;

    let mut is_hovered = use_signal(|| false);
    rsx! {
        g {
            circle {
                cx: "{position.x}",
                cy: "{position.y - PORT_RADIUS}",
                r: "{PORT_RADIUS}",
                fill: if *is_hovered.read() { IoPortStyleUrl::HOVER } else { IoPortStyleUrl::GRADIENT },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1.5",
                onmouseenter: move |_| is_hovered.set(true),
                onmouseleave: move |_| is_hovered.set(false),
                onmouseup: move |_| { use_context::<Handlers>().on_mouse_up.call(props.id) },
            }

            circle {
                cx: "{position.x}",
                cy: "{position.y - PORT_RADIUS}",
                r: "2.5",
                fill: "rgba(255,255,255,0.4)",
                pointer_events: "none",
            }
        }
    }
}

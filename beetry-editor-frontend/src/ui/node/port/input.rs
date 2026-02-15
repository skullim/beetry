use crate::{
    Point,
    ui::{handler::define_handlers, node::port::IoPortStyleUrl},
};
use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;

define_handlers!(on_mouse_up: NodeId);

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
    let handlers = use_context::<Handlers>();
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
                onmouseup: move |_| { handlers.on_mouse_up.call(props.id) },
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

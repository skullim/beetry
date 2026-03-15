use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;

use crate::{
    Point,
    ui::{handler::define_handlers, node::pin::IoPinStyleUrl},
};

define_handlers!(on_mouse_up: NodeId);

#[derive(Props, PartialEq, Clone)]
pub struct PinProps {
    id: NodeId,
    position: Point,
}

#[component]
pub(crate) fn Pin(props: PinProps) -> Element {
    const PIN_RADIUS: f64 = 7.0;
    let position = props.position;

    let mut is_hovered = use_signal(|| false);
    let handlers = use_context::<Handlers>();
    rsx! {
        g {
            circle {
                cx: "{position.x}",
                cy: "{position.y - PIN_RADIUS}",
                r: "{PIN_RADIUS}",
                fill: if *is_hovered.read() { IoPinStyleUrl::HOVER } else { IoPinStyleUrl::GRADIENT },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1.5",
                onmouseenter: move |_| is_hovered.set(true),
                onmouseleave: move |_| is_hovered.set(false),
                onmouseup: move |_| { handlers.on_mouse_up.call(props.id) },
            }

            circle {
                cx: "{position.x}",
                cy: "{position.y - PIN_RADIUS}",
                r: "2.5",
                fill: "rgba(255,255,255,0.4)",
                pointer_events: "none",
            }
        }
    }
}

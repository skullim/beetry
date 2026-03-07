use crate::{
    Point,
    ui::{handler::define_handlers, node::pin::IoPinStyleUrl},
};
use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;

use crate::definitions::IndexedDragOffset;

define_handlers!(on_mouse_down: IndexedDragOffset);

#[derive(Props, PartialEq, Clone)]
pub struct PinProps {
    id: NodeId,
    position: Point,
}

#[component]
pub(crate) fn Pin(props: PinProps) -> Element {
    static PIN_RADIUS: f64 = 7.0;

    let position = props.position;
    let mut is_hovered = use_signal(|| false);
    let handlers = use_context::<Handlers>();
    rsx! {
        g {
            circle {
                cx: "{position.x}",
                cy: "{position.y + PIN_RADIUS}",
                r: "{PIN_RADIUS}",
                fill: if *is_hovered.read() { IoPinStyleUrl::HOVER } else { IoPinStyleUrl::GRADIENT },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1.5",
                onmouseenter: move |_| is_hovered.set(true),
                onmouseleave: move |_| is_hovered.set(false),
                onmousedown: move |evt| {
                    evt.stop_propagation();
                    let mouse_coords = evt.element_coordinates();
                    let offset = Point {
                        x: mouse_coords.x,
                        y: mouse_coords.y,
                    };
                    handlers
                        .on_mouse_down
                        .call(IndexedDragOffset {
                            id: props.id,
                            offset,
                        });
                },
            }

            circle {
                cx: "{position.x}",
                cy: "{position.y + PIN_RADIUS}",
                r: "2.5",
                fill: "rgba(255,255,255,0.4)",
                pointer_events: "none",
            }
        }
    }
}

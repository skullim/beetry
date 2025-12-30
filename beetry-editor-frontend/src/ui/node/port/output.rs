use crate::{Point, ui::node::port::IoPortStyleUrl};
use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;

use crate::definitions::IndexedDragOffset;

#[derive(Debug, Clone)]
pub struct Handlers {
    pub(crate) on_mouse_down: EventHandler<IndexedDragOffset>,
}

impl Handlers {
    pub(crate) fn new(on_mouse_down: impl FnMut(IndexedDragOffset) + 'static) -> Self {
        Self {
            on_mouse_down: EventHandler::new(on_mouse_down),
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
                cy: "{position.y + PORT_RADIUS}",
                r: "{PORT_RADIUS}",
                fill: if *is_hovered.read() { IoPortStyleUrl::HOVER } else { IoPortStyleUrl::GRADIENT },
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
                    use_context::<Handlers>()
                        .on_mouse_down
                        .call(IndexedDragOffset {
                            id: props.id,
                            offset,
                        })
                },
            }

            circle {
                cx: "{position.x}",
                cy: "{position.y + PORT_RADIUS}",
                r: "2.5",
                fill: "rgba(255,255,255,0.4)",
                pointer_events: "none",
            }
        }
    }
}

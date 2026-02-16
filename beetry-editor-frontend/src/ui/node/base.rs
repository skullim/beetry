use crate::Point;
use crate::ui::handler::define_handlers;
use crate::ui::node::tooltip::Tooltip;
use crate::ui::{shadow, text};
use beetry_editor_types::id::NodeId;
use bon::Builder;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use std::rc::Rc;

define_handlers!(on_menu: (NodeId, Point),
          on_mouse_down: (NodeId, Point, Event<MouseData>),
);

#[derive(Clone, Debug, PartialEq, Builder)]
pub(super) struct NodeStyle {
    #[builder(into)]
    pub fill_gradient: String,
    #[builder(into)]
    pub hover_gradient: String,
    #[builder(into)]
    pub label: String,
    #[builder(default = 100.0)]
    pub width: f64,
    #[builder(default = 100.0)]
    pub height: f64,
}

#[derive(Props, Clone, PartialEq)]
pub struct NodeBaseProps {
    id: NodeId,
    position: Point,
    style: Rc<NodeStyle>,
}

#[component]
pub fn NodeWithMenu(children: Element, id: NodeId) -> Element {
    let menu_handler = use_context::<Handlers>().on_menu;
    rsx! {
        g {
            oncontextmenu: move |evt| {
                if evt.held_buttons().contains(MouseButton::Secondary) {
                    evt.prevent_default();
                    let mouse_coords = evt.element_coordinates();
                    menu_handler
                        .call((
                            id,
                            Point {
                                x: mouse_coords.x,
                                y: mouse_coords.y,
                            },
                        ));
                }
            },
            {children}
        }
    }
}

#[component]
pub fn NodeBase(props: NodeBaseProps) -> Element {
    let position = props.position;
    let id = props.id;
    let style = &props.style;

    let mut is_hovered = use_signal(|| false);
    let fill_color = if *is_hovered.peek() {
        &style.hover_gradient
    } else {
        &style.fill_gradient
    };

    let handlers = use_context::<Handlers>();

    rsx! {
        g {
            onmousedown: move |evt| {
                if evt.held_buttons().contains(MouseButton::Primary) {
                    handlers.on_mouse_down.call((id, position, evt))
                }
            },
            onmouseenter: move |_| is_hovered.set(true),
            onmouseleave: move |_| is_hovered.set(false),
            style: "cursor: grab;",

            rect {
                x: "{position.x}",
                y: "{position.y}",
                width: "{style.width}",
                height: "{style.height}",
                fill: "{fill_color}",
                filter: if *is_hovered.read() { shadow::FilterUrl::SHADOW_HOVER } else { shadow::FilterUrl::SHADOW },
                stroke: "rgba(255,255,255,0.2)",
                stroke_width: "1.5",
                rx: "12",
                ry: "12",
            }

            text {
                pointer_events: "none",
                x: "{position.x + style.width / 2.0}",
                y: "{position.y + style.height / 2.0}",
                text_anchor: "middle",
                dominant_baseline: "middle",
                font_family: text::font_family(),
                font_size: "{text::FONT_SIZE_NORMAL}",
                font_weight: "semi-bold",
                fill: "white",
                "{style.label}"
            }

            Tooltip {
                visible: is_hovered,
                node_id: id,
                anchor: Point {
                    x: position.x + style.width + 8.0,
                    y: position.y,
                },
            }
        }
    }
}

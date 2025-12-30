use crate::Point;
use crate::definitions::IndexedDragOffset;
use crate::ui::viewport::{ViewportContext, ZoomLevel};
use crate::ui::{shadow, text};
use beetry_editor_types::id::NodeId;
use bon::Builder;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Handlers {
    on_drag_start: EventHandler<IndexedDragOffset>,
    on_context_menu: EventHandler<(NodeId, Point)>,
}

impl Handlers {
    pub(crate) fn new(
        on_drag_start: impl FnMut(IndexedDragOffset) + 'static,
        on_context_menu: impl FnMut((NodeId, Point)) + 'static,
    ) -> Self {
        Self {
            on_drag_start: EventHandler::new(on_drag_start),
            on_context_menu: EventHandler::new(on_context_menu),
        }
    }
}

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
pub fn NodeWithContextMenu(children: Element, id: NodeId) -> Element {
    let context_menu_handler = use_context::<Handlers>().on_context_menu;
    rsx! {
        g {
            oncontextmenu: move |evt| {
                if evt.held_buttons().contains(MouseButton::Secondary) {
                    evt.prevent_default();
                    let mouse_coords = evt.client_coordinates();
                    context_menu_handler
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

    let on_drag_start_cb = use_context::<Handlers>().on_drag_start;
    let zoom_level = use_context::<ViewportContext>().zoom_level;

    let mut is_hovered = use_signal(|| false);
    let fill_color = if *is_hovered.peek() {
        &style.hover_gradient
    } else {
        &style.fill_gradient
    };

    rsx! {
        g {
            onmousedown: move |evt| {
                evt.stop_propagation();
                on_mouse_down(evt, position, id, zoom_level.into(), &on_drag_start_cb);
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
                font_size: "12",
                font_weight: "semi-bold",
                fill: "white",
                "{style.label}"
            }
        }
    }
}

fn on_mouse_down(
    evt: Event<MouseData>,
    position: Point,
    id: NodeId,
    zoom_level: ReadSignal<ZoomLevel>,
    drag_start_cb: &Callback<IndexedDragOffset>,
) {
    if evt.held_buttons().contains(MouseButton::Primary) {
        let mouse_coords = evt.client_coordinates();
        let zoom_level = zoom_level.peek().get();

        let svg_mouse_coords = Point {
            x: mouse_coords.x / zoom_level,
            y: mouse_coords.y / zoom_level,
        };

        let drag_offset = Point {
            x: svg_mouse_coords.x - position.x,
            y: svg_mouse_coords.y - position.y,
        };
        drag_start_cb.call(IndexedDragOffset {
            id,
            offset: drag_offset,
        });
    }
}

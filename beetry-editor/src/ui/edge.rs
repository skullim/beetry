mod context_menu;
mod renderer;
mod tracker;

pub mod temporary;
pub use context_menu::{ContextMenu, Handlers as ContextMenuHandlers, State as ContextMenuState};
pub use renderer::Renderer;
pub use temporary::Temporary;
pub use tracker::Tracker;

use dioxus::prelude::*;

use crate::{
    definitions::{Point, PointEdge},
    ui::curve::Curve,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context {
    pub(crate) tracker: Signal<Tracker>,
}

impl Context {
    pub(crate) fn new() -> Self {
        Self {
            tracker: Signal::new(Tracker::new()),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct EdgeProps {
    edge: PointEdge,
    edge_index: usize,
    on_context_menu: EventHandler<(usize, Point)>,
}

#[component]
pub(crate) fn Edge(props: EdgeProps) -> Element {
    let start = props.edge.start;
    let end = props.edge.end;
    let edge_index = props.edge_index;

    let curve_start = Point {
        x: start.x + 50.0, // Center of node + offset to output port
        y: start.y + 70.0,
    };

    let curve_end = Point {
        x: end.x + 50.0, // Center of node + offset to input port,
        y: end.y - 10.0, // Top of node (input port),
    };

    let stroke_color = "#8B5CF6"; // matches output ports
    let stroke_width = "3";

    rsx! {
        path {
            d: "{Curve::calculate_vertical(&curve_start, &curve_end)}",
            stroke: stroke_color,
            stroke_width,
            fill: "none",
            cursor: "pointer",
            oncontextmenu: move |evt| {
                evt.prevent_default();
                let click_point = Point {
                    x: evt.page_coordinates().x,
                    y: evt.page_coordinates().y,
                };
                props.on_context_menu.call((edge_index, click_point));
            },
        }
    }
}

mod context_menu;
mod renderer;

pub mod temporary;
use beetry_editor_types::EdgeId;
pub use context_menu::{ContextMenu, Handlers as ContextMenuHandlers, State as ContextMenuState};
pub use renderer::Renderer2;
pub use temporary::Temporary;

use dioxus::prelude::*;

use crate::definitions::{EdgePos, Point};
use crate::ui::curve::Curve;

#[derive(Props, Clone, PartialEq)]
pub struct EdgeProps {
    pos: EdgePos,
    edge_index: EdgeId,
    on_context_menu: EventHandler<(usize, Point)>,
}

#[component]
pub(crate) fn Edge(props: EdgeProps) -> Element {
    let start = props.pos.start;
    let end = props.pos.end;
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

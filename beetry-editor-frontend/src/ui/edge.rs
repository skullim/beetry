pub mod menu;
pub mod renderer;
pub mod temporary;

use beetry_editor_types::id::EdgeId;
use dioxus::prelude::*;
pub use menu::Menu;
pub use renderer::Renderer;
pub use temporary::Temporary;

use crate::{
    Point,
    definitions::EdgePos,
    ui::{handler::define_handlers, style::curve::Curve},
};

define_handlers!(on_menu: (EdgeId, Point));

#[derive(Props, Clone, PartialEq)]
pub struct EdgeProps {
    pos: EdgePos,
    edge_id: EdgeId,
}

#[component]
pub(crate) fn Edge(props: EdgeProps) -> Element {
    let start = props.pos.start;
    let end = props.pos.end;
    let edge_id = props.edge_id;

    let curve_start = Point {
        x: start.x + 50.0, // Center of node + offset to output pin
        y: start.y + 70.0,
    };

    let curve_end = Point {
        x: end.x + 50.0, // Center of node + offset to input pin,
        y: end.y - 10.0, // Top of node (input pin),
    };

    let stroke_color = "#8B5CF6"; // matches output pins
    let stroke_width = "3";

    let handlers = use_context::<Handlers>();

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
                    x: evt.element_coordinates().x,
                    y: evt.element_coordinates().y,
                };
                handlers.on_menu.call((edge_id, click_point));
            },
        }
    }
}

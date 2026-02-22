use crate::Point;
use crate::definitions::EdgePos;
use crate::ui::channel::edge_menu::ConnectionId;
use crate::ui::curve::Curve;
use crate::ui::handler::define_handlers;
use dioxus::prelude::*;

define_handlers!(on_menu: (ConnectionId, Point));

impl Handlers {
    pub(crate) fn on_menu_handler(&self) -> EventHandler<(ConnectionId, Point)> {
        self.on_menu
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct EdgeProps {
    pub pos: EdgePos,
    pub connection: ConnectionId,
    pub stroke: &'static str,
}

#[component]
pub fn Edge(props: EdgeProps) -> Element {
    let on_menu = use_context::<Handlers>().on_menu_handler();

    rsx! {
        path {
            d: "{Curve::calculate_horizontal(&props.pos.start, &props.pos.end)}",
            stroke: props.stroke,
            stroke_width: "3",
            fill: "none",
            cursor: "pointer",
            oncontextmenu: move |evt| {
                evt.prevent_default();
                evt.stop_propagation();
                let click_point = Point {
                    x: evt.element_coordinates().x,
                    y: evt.element_coordinates().y,
                };
                on_menu.call((props.connection, click_point));
            },
        }
    }
}

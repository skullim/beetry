mod context;
pub(crate) mod handlers;
mod state;

pub(crate) use state::State;

use beetry_editor_types::output::ui::Point;
use dioxus::logger::tracing::debug;
use dioxus::prelude::*;

use crate::signals::RenderRequests;
use crate::ui::channel::{self};
use crate::ui::handler::define_handlers;
use crate::ui::node;
use crate::ui::{self, edge};

define_handlers!(
    on_mouse_move: Event<MouseData>,
    on_mouse_up: Event<MouseData>,
    on_wheel: Event<WheelData>,
    on_scroll: Event<ScrollData>,
);

#[component]
pub(crate) fn Workspace(
    render_requests: RenderRequests,
    element_spawn_point: Signal<Point>,
    parameter_state: Signal<node::parameter::State>,
) -> Element {
    let state = State::new();
    rsx! {
        context::Provider {
            state,
            render_requests,
            element_spawn_point,
            parameter_state,

            Canvas {
                state,
                render_requests,
            }
        }
    }
}

#[component]
fn Canvas(state: State, render_requests: RenderRequests) -> Element {
    debug!("rendering workspace");

    let handlers = use_context::<Handlers>();
    let zoom_level = *state.svg.zoom.read();
    let dimensions = state.svg.dimensions;
    let menus = state.menu;

    rsx! {
        div {
            style: "overflow: auto; border: 1px solid black; width: 800px; height: 800px;",
            onwheel: handlers.on_wheel,
            onscroll: handlers.on_scroll,

            svg {
                style: "transform: scale({zoom_level}); transform-origin: 0 0;",
                width: "{dimensions.width()}",
                height: "{dimensions.height()}",

                onmousemove: handlers.on_mouse_move,
                onmouseup: handlers.on_mouse_up,

                rect {
                    x: "0",
                    y: "0",
                    width: "100%",
                    height: "100%",
                    fill: "#5045454f",
                }

                {ui::channel::style_defs()}
                {ui::node::style_defs()}
                {ui::shadow::style_defs()}

                {grid_style_defs()}
                rect {
                    x: "0",
                    y: "0",
                    width: "100%",
                    height: "100%",
                    fill: "url(#grid)",
                }

                edge::Renderer { render_edges: render_requests.edges }
                channel::ConnectionRenderer { render_channel_edges: render_requests.channel_edges }
                channel::Renderer { render_channels: render_requests.channels }
                node::Renderer { render_nodes: render_requests.nodes }

                edge::Temporary {
                    state: use_memo(move || (&*state.temp.edge.read()).into()),
                    orientation: edge::temporary::CurveOrientation::Vertical,
                    stroke: "#A78BFA",
                }
                edge::Temporary {
                    state: use_memo(move || (&*state.temp.channel.read()).into()),
                    orientation: edge::temporary::CurveOrientation::Horizontal,
                    stroke: "#3a2020ff",
                }

                node::Menu { state: menus.node }
                edge::Menu { state: menus.edge }
                channel::Menu { state: menus.channel }
                node::port::Menu { state: menus.port }
            }
        }
    }
}

pub(crate) fn grid_style_defs() -> Element {
    rsx! {
        defs {
            pattern {
                id: "grid",
                width: "50",
                height: "50",
                pattern_units: "userSpaceOnUse",

                path {
                    d: "M 50 0 L 0 0 0 50",
                    fill: "none",
                    stroke: "#d0d0d0",
                    stroke_width: "2",
                }
            }
        }
    }
}

mod context;
pub(crate) mod handlers;
mod state;

pub(crate) use state::State;

use dioxus::logger::tracing::debug;
use dioxus::prelude::*;

use crate::components::editor;
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
pub(crate) fn Workspace(render_requests: RenderRequests, editor_state: editor::State) -> Element {
    let state = State::new();
    rsx! {
        context::Provider { state, editor_state, render_requests,
            Canvas { state, editor_state, render_requests }
        }
    }
}

#[component]
fn Canvas(state: State, editor_state: editor::State, render_requests: RenderRequests) -> Element {
    debug!("rendering workspace");

    let handlers = use_context::<Handlers>();
    let zoom_level = *state.svg.zoom.read();
    let dimensions = state.svg.dimensions;
    let menus = state.menu;

    rsx! {
        div {
            class: "bt-workspace-canvas",
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
                    fill: "#1a2a46",
                }

                {ui::channel::style_defs()}
                {ui::node::style_defs()}
                {ui::shadow::style_defs()}

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
                    stroke: "rgb(167, 162, 162)",
                }

                node::Menu { state: menus.node }
                edge::Menu { state: menus.edge }
                channel::Menu { state: menus.channel }
                channel::edge_menu::Menu { state: menus.channel_edge }
                node::port::Menu { state: menus.port }
            }

            node::parameter::Dialog { state: editor_state.parameter }
            channel::dialog::Dialog { state: editor_state.channel_dialog_state }
        }
    }
}

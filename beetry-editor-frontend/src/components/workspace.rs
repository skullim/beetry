pub(crate) mod providers;
mod state;

use beetry_editor_types::output::ui::Point;
use dioxus::logger::tracing::debug;
use dioxus::prelude::*;

use crate::components::workspace::providers::WorkspaceCtx;
use crate::signals::RenderRequests;
use crate::ui::channel::{self};
use crate::ui::node;
use crate::ui::{self, edge};

#[component]
pub(crate) fn Workspace(
    render_requests: RenderRequests,
    ui_spawn_point: Signal<Point>,
    parameter_dialog_state: Signal<node::ParameterDialogState>,
) -> Element {
    rsx! {
        WorkspaceContextProvider {
            render_requests,
            parameter_dialog_state,

            WorkspaceCanvas {
                render_requests,
                ui_spawn_point,
            }
        }
    }
}

#[component]
fn WorkspaceContextProvider(
    render_requests: RenderRequests,
    parameter_dialog_state: Signal<node::ParameterDialogState>,
    children: Element,
) -> Element {
    let backend = use_context();
    let workspace_ctx = use_context_provider(|| WorkspaceCtx::new(backend, render_requests));

    let error_queue_state = use_context();
    use_context_provider(|| {
        providers::node::handlers(
            workspace_ctx.state.drag,
            workspace_ctx.state.menus,
            workspace_ctx.state.svg,
            workspace_ctx.backend,
            error_queue_state,
        )
    });
    use_context_provider(|| {
        providers::port::input_handlers(
            workspace_ctx.state.temp,
            workspace_ctx.backend,
            workspace_ctx.requests,
        )
    });
    use_context_provider(|| providers::port::output_handlers(workspace_ctx.state.temp));
    use_context_provider(|| {
        providers::node::context_menu_handlers(
            workspace_ctx.state.menus,
            workspace_ctx.backend,
            workspace_ctx.requests,
            parameter_dialog_state,
        )
    });

    use_context_provider(|| providers::edge::handlers(workspace_ctx.state.menus));
    use_context_provider(|| {
        providers::edge::context_menu_handlers(
            workspace_ctx.state.menus,
            workspace_ctx.backend,
            workspace_ctx.requests,
        )
    });
    use_context_provider(|| {
        providers::channel::context_menu_handlers(
            workspace_ctx.state.menus,
            workspace_ctx.backend,
            workspace_ctx.requests,
        )
    });

    use_context_provider(|| {
        providers::port::sender_handlers(
            workspace_ctx.state.menus,
            workspace_ctx.state.temp,
            workspace_ctx.backend,
            error_queue_state,
        )
    });
    use_context_provider(|| {
        providers::port::receiver_handlers(
            workspace_ctx.state.menus,
            workspace_ctx.state.temp,
            workspace_ctx.backend,
            error_queue_state,
        )
    });
    use_context_provider(|| {
        providers::port::context_menu_handlers(workspace_ctx.backend, workspace_ctx.requests)
    });

    use_context_provider(|| {
        providers::channel::handlers(
            workspace_ctx.state.drag,
            workspace_ctx.state.menus,
            workspace_ctx.state.temp,
            workspace_ctx.state.svg,
            workspace_ctx.backend,
            workspace_ctx.requests,
            error_queue_state,
        )
    });

    children
}

#[component]
fn WorkspaceCanvas(render_requests: RenderRequests, ui_spawn_point: Signal<Point>) -> Element {
    debug!("rendering workspace");

    let ws = use_context::<providers::WorkspaceCtx>();
    let handlers = ws.workspace_handlers;
    let zoom_level = *ws.state.svg.zoom.read();
    let dimensions = ws.state.svg.dimensions;
    let (menus, temp) = (ws.state.menus, ws.state.temp);

    rsx! {
        div {
            style: "overflow: auto; border: 1px solid black; width: 800px; height: 800px;",
            onwheel: handlers.on_wheel,
            onscroll: move |evt| {
                let x = evt.scroll_left() / zoom_level;
                let y = evt.scroll_top() / zoom_level;
                ui_spawn_point.set(Point { x, y });
            },

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

                {providers::render_request::grid_style_defs()}
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

                edge::Temporary { state: use_memo(move || (&*ws.state.temp.edge.read()).into()) }
                if temp.channel.is_dragged() {
                    channel::Temporary { edge: temp.channel.edge() }
                }

                node::ContextMenu { state: menus.node }
                edge::ContextMenu { state: menus.edge }
                channel::ContextMenu { state: menus.channel }
                node::port_context_menu::Menu { state: menus.port }
            }
        }
    }
}

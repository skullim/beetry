use beetry_editor_backend::node::PortConnectionInput;
use beetry_editor_types::{
    id::{ChannelId, EdgeId, NodeId, NodePortId},
    output::edge::NodeEdge,
    output::ui::{ChannelPosition, NodePosition, Point},
};
use bon::Builder;
use dioxus::logger::tracing::debug;
use dioxus::prelude::*;

use crate::definitions::{EdgePos, IndexedDragOffset};
use crate::editor::ServiceContext;
use crate::signals::RequestRender;
use crate::ui::channel::temporary::{ConnectionOrigin, DraggedData};
use crate::ui::channel::{self};
use crate::ui::node::{self, ContextMenuState, ReceiverPortHandlers, SenderPortHandlers};
use crate::ui::viewport::ViewportContext;
use crate::ui::{self, edge};

#[derive(Debug, Clone, Copy)]
struct DimensionsContext {
    width: Signal<f64>,
    height: Signal<f64>,
}

impl DimensionsContext {
    const DEFAULT_SIZE: f64 = 1600.0;
    const MARGIN: f64 = 400.0;

    fn new() -> Self {
        Self {
            width: Signal::new(Self::DEFAULT_SIZE),
            height: Signal::new(Self::DEFAULT_SIZE),
        }
    }

    fn width(&self) -> f64 {
        *self.width.read()
    }

    fn height(&self) -> f64 {
        *self.height.read()
    }

    fn resize_if_needed<'a>(&mut self, positions: impl Iterator<Item = &'a NodePosition>) {
        let (new_width, new_height) = positions
            .map(|node| (node.origin.x + Self::MARGIN, node.origin.y + Self::MARGIN))
            .fold(
                (Self::DEFAULT_SIZE, Self::DEFAULT_SIZE),
                |(width, height), (x, y)| (width.max(x), height.max(y)),
            );

        self.width.set(new_width);
        self.height.set(new_height);
    }
}

#[derive(Debug, Clone, Copy)]
enum DragNodeState {
    Idle,
    Dragged { id: NodeId, offset: Point },
}

#[derive(Debug, Clone, Copy)]
enum DragChannelState {
    Idle,
    Dragged { id: ChannelId, offset: Point },
}

#[derive(Debug, Clone, Builder)]
struct WorkspaceEventHandlers {
    on_mouse_move: EventHandler<Event<MouseData>>,
    on_mouse_up: EventHandler<Event<MouseData>>,
    on_wheel: EventHandler<Event<WheelData>>,
}

fn style_defs() -> Element {
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

#[derive(Debug, Clone)]
pub struct WorkspaceContext {
    dimensions_ctx: DimensionsContext,
    drag_node_state: Signal<DragNodeState>,
    drag_channel_state: Signal<DragChannelState>,
    context_menu_state: Signal<ContextMenuState>,
    edge_context_menu_state: Signal<edge::ContextMenuState>,
    channel_temp_conn_ctx: channel::temporary::Context,
}

impl WorkspaceContext {
    fn new() -> Self {
        Self {
            dimensions_ctx: DimensionsContext::new(),
            drag_node_state: Signal::new(DragNodeState::Idle),
            drag_channel_state: Signal::new(DragChannelState::Idle),
            context_menu_state: Signal::new(node::ContextMenuState::default()),
            edge_context_menu_state: Signal::new(edge::ContextMenuState::default()),
            channel_temp_conn_ctx: channel::temporary::Context::new(),
        }
    }
}

#[component]
pub(crate) fn Workspace(
    render_nodes: Signal<RequestRender>,
    render_channels: Signal<RequestRender>,
) -> Element {
    debug!("rendering workspace");

    let workspace_ctx = use_context_provider(WorkspaceContext::new);

    let mut render_edges = use_signal(RequestRender::new);
    let mut drag_node_state = workspace_ctx.drag_node_state;
    let mut drag_channel_state = workspace_ctx.drag_channel_state;
    let node_context_menu_state = workspace_ctx.context_menu_state;
    let edge_context_menu_state = workspace_ctx.edge_context_menu_state;
    let mut dimensions_ctx = workspace_ctx.dimensions_ctx;
    let mut temp_channel_conn_ctx = workspace_ctx.channel_temp_conn_ctx;

    let mut temp_edge_ctx = use_context_provider(edge::temporary::Context::new);
    let mut viewport_ctx = use_context_provider(ViewportContext::new);
    let zoom_level = viewport_ctx.zoom_level;

    use_context_provider(move || node_handlers(drag_node_state, node_context_menu_state));
    use_context_provider(move || input_port_handlers(temp_edge_ctx, render_edges));
    use_context_provider(move || output_port_handlers(temp_edge_ctx));
    use_context_provider(move || {
        node_context_menu_handlers(node_context_menu_state, render_nodes, render_edges)
    });
    use_context_provider(move || edge_context_menu_handlers(edge_context_menu_state, render_edges));

    use_context_provider(move || sender_handlers(temp_channel_conn_ctx));
    use_context_provider(move || receiver_handlers(temp_channel_conn_ctx));
    use_context_provider(move || {
        channel_handlers(drag_channel_state, temp_channel_conn_ctx, render_channels)
    });

    let workspace_handlers_ctx = use_context_provider(move || {
        let on_mouse_move = move |evt: Event<MouseData>| {
            evt.stop_propagation();
            if let DragNodeState::Dragged { id, offset } = *drag_node_state.peek() {
                let mouse_coords = evt.client_coordinates();

                let zoom = zoom_level.peek().get();
                let updated_pos = Point {
                    x: (mouse_coords.x / zoom - offset.x),
                    y: (mouse_coords.y / zoom - offset.y),
                };
                let mut service = use_context::<ServiceContext>();
                {
                    let mut write = service.service.write();
                    let mut ui_api = write.ui_api_mut();
                    //@todo error handling
                    ui_api.node().update_position(
                        id,
                        NodePosition {
                            origin: updated_pos,
                        },
                    );
                }

                render_nodes.with_mut(|write| write.request());
                render_edges.with_mut(|write| write.request());
                //@todo actually just channel edges if they are separated
                render_channels.with_mut(|write| write.request());

                let read = service.service.read();
                let ui_api = read.ui_api();
                let ui_node_api = ui_api.node();
                let positions_iter = ui_node_api.positions();
                dimensions_ctx.resize_if_needed(positions_iter);
            }

            if let DragChannelState::Dragged { id, offset } = *drag_channel_state.peek() {
                let mouse_coords = evt.client_coordinates();

                let zoom = zoom_level.peek().get();
                let updated_pos = Point {
                    x: (mouse_coords.x / zoom - offset.x),
                    y: (mouse_coords.y / zoom - offset.y),
                };

                let mut service = use_context::<ServiceContext>();
                let mut write = service.service.write();
                let mut ui_api = write.ui_api_mut();
                //@todo error handling
                ui_api.channel().update_position(
                    id,
                    ChannelPosition {
                        origin: updated_pos,
                    },
                );
                render_channels.with_mut(|write| write.request());
            }

            temp_edge_ctx.update_end_if_dragged(&evt);
            temp_channel_conn_ctx.update_end_if_dragged(&evt);
        };

        let on_mouse_up = move |evt: Event<MouseData>| {
            evt.stop_propagation();
            drag_node_state.set(DragNodeState::Idle);
            drag_channel_state.set(DragChannelState::Idle);
            temp_edge_ctx.reset();
            temp_channel_conn_ctx.reset();
        };

        let on_wheel = move |evt: Event<WheelData>| {
            if evt.modifiers().ctrl() {
                evt.prevent_default();
                let delta = evt.delta();
                viewport_ctx
                    .zoom_level
                    .with_mut(|level| level.update(&delta));
            }
        };

        WorkspaceEventHandlers::builder()
            .on_mouse_move(EventHandler::new(on_mouse_move))
            .on_mouse_up(EventHandler::new(on_mouse_up))
            .on_wheel(EventHandler::new(on_wheel))
            .build()
    });

    rsx! {
        div {
            style: "overflow: auto; border: 1px solid black; width: 800px; height: 800px;",
            onwheel: workspace_handlers_ctx.on_wheel,

            svg {
                style: "transform: scale({zoom_level.read().get()}); transform-origin: 0 0;",
                width: "{dimensions_ctx.width()}",
                height: "{dimensions_ctx.height()}",

                onmousemove: workspace_handlers_ctx.on_mouse_move,
                onmouseup: workspace_handlers_ctx.on_mouse_up,

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

                {style_defs()}
                rect {
                    x: "0",
                    y: "0",
                    width: "100%",
                    height: "100%",
                    fill: "url(#grid)",
                }

                edge::Renderer { render_edges, edge_context_menu_state }
                channel::Renderer { render_channels }
                node::Renderer { render_nodes }

                if temp_edge_ctx.is_dragged() {

                    edge::Temporary { edge: temp_edge_ctx.edge() }
                }
                if temp_channel_conn_ctx.is_dragged() {
                    channel::Temporary { edge: temp_channel_conn_ctx.edge() }
                }
            }

            node::ContextMenu { state: node_context_menu_state }
            edge::ContextMenu { state: edge_context_menu_state }
        }
    }
}

fn input_port_handlers(
    mut temp_edge_ctx: edge::temporary::Context,
    mut render_edges: Signal<RequestRender>,
) -> node::InputPortHandlers {
    let on_mouse_up = move |to: NodeId| {
        if let Some(from) = temp_edge_ctx.take_dragged()
            && from != to
        {
            let mut service = use_context::<ServiceContext>();
            let mut write = service.service.write();
            let mut edge_api = write.edge_api_mut();
            //@todo error handling
            edge_api.create(NodeEdge { from, to }).unwrap();
            render_edges.with_mut(|write| write.request());
            debug!("created edge from node {from}: to: {to}");
        }
    };
    node::InputPortHandlers::new(on_mouse_up)
}

fn output_port_handlers(mut temp_edge_ctx: edge::temporary::Context) -> node::OutputPortHandlers {
    let on_mouse_down = move |indexed_drag_offset: IndexedDragOffset| {
        let offset = indexed_drag_offset.offset;
        temp_edge_ctx.set_dragged_from(indexed_drag_offset.id);
        temp_edge_ctx.update_edge(EdgePos {
            start: offset,
            end: offset,
        });
    };
    node::OutputPortHandlers::new(on_mouse_down)
}

fn node_handlers(
    mut dragged_node: Signal<DragNodeState>,
    mut ctx_menu_state: Signal<node::ContextMenuState>,
) -> node::Handlers {
    let on_drag_start = move |indexed_offset: IndexedDragOffset| {
        dragged_node.set(DragNodeState::Dragged {
            id: indexed_offset.id,
            offset: indexed_offset.offset,
        });
    };

    let on_context_menu = move |(node_id, position): (NodeId, Point)| {
        ctx_menu_state.set(node::ContextMenuState {
            position,
            target_node: node_id,
            is_visible: true,
        });
    };
    node::Handlers::new(on_drag_start, on_context_menu)
}

fn sender_handlers(
    mut channel_temp_connection_ctx: channel::temporary::Context,
) -> SenderPortHandlers {
    let on_mouse_down = move |(origin, indexed_drag_offset, port_id): (
        ConnectionOrigin,
        IndexedDragOffset,
        NodePortId,
    )| {
        let dragged_data = DraggedData {
            node_id: indexed_drag_offset.id,
            origin,
            port_id,
        };
        let offset = indexed_drag_offset.offset;
        channel_temp_connection_ctx.set_dragged(dragged_data);
        channel_temp_connection_ctx.update_edge_pos(EdgePos {
            start: offset,
            end: offset,
        });
    };
    SenderPortHandlers::new(on_mouse_down)
}

fn receiver_handlers(
    mut channel_temp_connection_ctx: channel::temporary::Context,
) -> ReceiverPortHandlers {
    let on_mouse_down = move |(origin, indexed_drag_offset, port_id): (
        ConnectionOrigin,
        IndexedDragOffset,
        NodePortId,
    )| {
        let dragged_data = DraggedData {
            node_id: indexed_drag_offset.id,
            origin,
            port_id,
        };
        let offset = indexed_drag_offset.offset;
        channel_temp_connection_ctx.set_dragged(dragged_data);
        channel_temp_connection_ctx.update_edge_pos(EdgePos {
            start: offset,
            end: offset,
        });
    };

    let on_context_menu = move |(node_id, port_id): (NodeId, NodePortId)| {
        //@todo enable setting external port. Service layer has to provide such API
    };

    ReceiverPortHandlers::new(on_mouse_down, on_context_menu)
}

fn channel_handlers(
    mut drag_channel_state: Signal<DragChannelState>,
    mut channel_temp_connection_ctx: channel::temporary::Context,
    mut render_channels: Signal<RequestRender>,
) -> channel::Handlers {
    let on_drag_start = move |(id, offset): (ChannelId, Point)| {
        drag_channel_state.set(DragChannelState::Dragged { id, offset });
    };

    let receiver_on_mouse_up = move |id: ChannelId| {
        if let Some(data) = channel_temp_connection_ctx.take_dragged()
            && matches!(data.origin, ConnectionOrigin::Receiver)
        {
            let mut service = use_context::<ServiceContext>();
            let mut write = service.service.write();
            let mut node_api_mut = write.node_api_mut();
            let mut port_connection = node_api_mut.port_connection();
            let input = PortConnectionInput::new(data.node_id, data.port_id, id);
            //@todo error handling
            port_connection.connect(input).unwrap();
            render_channels.with_mut(|write| write.request());
            info!(
                "connected channel {id} and node (id: {}, port_id: {})",
                data.node_id, data.port_id
            );
        }
    };

    let sender_on_mouse_up = move |id: ChannelId| {
        if let Some(data) = channel_temp_connection_ctx.take_dragged()
            && matches!(data.origin, ConnectionOrigin::Sender)
        {
            let mut service = use_context::<ServiceContext>();
            let mut write = service.service.write();
            let mut node_api_mut = write.node_api_mut();
            let mut port_connection = node_api_mut.port_connection();
            let input = PortConnectionInput::new(data.node_id, data.port_id, id);
            //@todo error handling
            port_connection.connect(input).unwrap();
            render_channels.with_mut(|write| write.request());
            info!(
                "connected channel {id} and node (id: {}, port_id: {})",
                data.node_id, data.port_id
            );
        }
    };

    channel::Handlers::new(on_drag_start, receiver_on_mouse_up, sender_on_mouse_up)
}

fn node_context_menu_handlers(
    mut node_ctx_menu_state: Signal<node::ContextMenuState>,
    mut render_nodes: Signal<RequestRender>,
    mut render_edges: Signal<RequestRender>,
) -> node::ContextMenuHandlers {
    let on_delete = move |node_id: NodeId| {
        let mut service = use_context::<ServiceContext>();
        let mut write = service.service.write();
        {
            let mut node_api = write.node_api_mut();
            let mut lifecycle = node_api.lifecycle();
            //@todo error handling
            lifecycle.remove(node_id);
        }
        let mut ui_api = write.ui_api_mut();
        //let mut ui_node_api = ui_api.node();
        ui_api.node().remove(node_id);

        node_ctx_menu_state.with_mut(|state| state.is_visible = false);
        render_nodes.with_mut(|write| write.request());
        render_edges.with_mut(|write| write.request());
        //@todo probably nodes should have been rerendered if nodes had present edges
    };

    let on_close = move |_| {
        node_ctx_menu_state.with_mut(|state| state.is_visible = false);
    };
    node::ContextMenuHandlers::new(on_delete, on_close)
}

fn edge_context_menu_handlers(
    mut edge_ctx_menu_state: Signal<edge::ContextMenuState>,
    mut render_edges: Signal<RequestRender>,
) -> edge::ContextMenuHandlers {
    let on_delete = move |edge_id: EdgeId| {
        let mut service = use_context::<ServiceContext>();
        let mut write = service.service.write();
        let mut edge_api = write.edge_api_mut();
        //@todo error handling
        edge_api.remove(edge_id).unwrap();
        edge_ctx_menu_state.with_mut(|state| state.is_visible = false);
        render_edges.with_mut(|write| write.request());
    };

    let on_close = move |()| {
        edge_ctx_menu_state.with_mut(|state| state.is_visible = false);
    };

    edge::ContextMenuHandlers::new(on_delete, on_close)
}

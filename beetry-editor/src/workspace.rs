use beetry_core::MessageHash;
use beetry_serde::de::channel::ChannelId;
use bon::Builder;
use dioxus::{logger::tracing::debug, prelude::*};

use crate::{
    definitions::{IndexedDragOffset, NodeEdge, NodeId, Point, PointEdge},
    ui::{
        self,
        channel::{self, temporary::ConnectionOrigin},
        edge,
        node::{self, ContextMenuState, ReceiverPortHandlers, SenderPortHandlers},
        viewport::ViewportContext,
    },
};

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceContext {
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

    fn resize_if_needed(&mut self, ui_nodes: ReadSignal<ui::NodeMap>) {
        let peeked_nodes = ui_nodes.peek();
        let (new_width, new_height) = peeked_nodes
            .values()
            .map(|node| (node.pos.x + Self::MARGIN, node.pos.y + Self::MARGIN))
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

#[component]
pub(crate) fn Workspace(
    ui_nodes: Signal<ui::NodeMap>,
    edge_ctx: edge::Context,
    channel_ctx: channel::Context,
) -> Element {
    debug!("rendering workspace");

    let workspace_ctx = use_context_provider(WorkspaceContext::new);
    let mut drag_node_state = workspace_ctx.drag_node_state;
    let mut drag_channel_state = workspace_ctx.drag_channel_state;
    let context_menu_state = workspace_ctx.context_menu_state;
    let edge_context_menu_state = workspace_ctx.edge_context_menu_state;
    let mut dimensions_ctx = workspace_ctx.dimensions_ctx;
    let mut temp_channel_conn_ctx = workspace_ctx.channel_temp_conn_ctx;

    let mut temp_edge_ctx = use_context_provider(edge::temporary::Context::new);
    let mut viewport_ctx = use_context_provider(ViewportContext::new);
    let zoom_level = viewport_ctx.zoom_level;

    use_context_provider(move || node_handlers(drag_node_state, context_menu_state));
    use_context_provider(move || input_port_handlers(ui_nodes.into(), temp_edge_ctx, edge_ctx));
    use_context_provider(move || output_port_handlers(temp_edge_ctx));
    use_context_provider(move || {
        context_menu_handlers(ui_nodes, context_menu_state, edge_ctx, channel_ctx)
    });
    use_context_provider(move || edge_context_menu_handlers(edge_context_menu_state, edge_ctx));

    use_context_provider(move || sender_handlers(temp_channel_conn_ctx));
    use_context_provider(move || receiver_handlers(temp_channel_conn_ctx, ui_nodes));
    use_context_provider(move || {
        channel_handlers(
            drag_channel_state,
            temp_channel_conn_ctx,
            channel_ctx.tracker,
        )
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

                ui_nodes.with_mut(|nodes| {
                    nodes.entry(id).and_modify(|e| e.pos = updated_pos);
                });

                dimensions_ctx.resize_if_needed(ui_nodes.into());
            }

            if let DragChannelState::Dragged { id, offset } = *drag_channel_state.peek() {
                let mouse_coords = evt.client_coordinates();

                let zoom = zoom_level.peek().get();
                let updated_pos = Point {
                    x: (mouse_coords.x / zoom - offset.x),
                    y: (mouse_coords.y / zoom - offset.y),
                };

                channel_ctx
                    .tracker
                    .with_mut(|tracker| tracker.update_channel_pose(id, updated_pos))
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

                edge::Renderer {
                    tracker: edge_ctx.tracker,
                    ui_nodes,
                    edge_context_menu_state,
                }
                channel::Renderer { ui_nodes, tracker: channel_ctx.tracker }
                node::Renderer { ui_nodes }

                if temp_edge_ctx.is_dragged() {

                    edge::Temporary { edge: temp_edge_ctx.edge() }
                }
                if temp_channel_conn_ctx.is_dragged() {
                    channel::Temporary { edge: temp_channel_conn_ctx.edge() }
                }
            }

            node::ContextMenu { state: context_menu_state }
            edge::ContextMenu { state: edge_context_menu_state }
        }
    }
}

fn would_create_cycle(tracker: &edge::Tracker, new_edge: &NodeEdge) -> bool {
    tracker.has_path(new_edge.to, new_edge.from)
}

fn input_port_handlers(
    ui_nodes: ReadSignal<ui::NodeMap>,
    mut temp_edge_ctx: edge::temporary::Context,
    mut edge_ctx: edge::Context,
) -> node::InputPortHandlers {
    let on_mouse_up = move |to: NodeId| {
        if let Some(from) = temp_edge_ctx.take_dragged()
            && from != to
        {
            let nodes = ui_nodes.peek();
            let from_node = nodes.get(&from).unwrap();
            let to_node = nodes.get(&to).unwrap();
            let edge = NodeEdge { from, to };

            {
                let tracker = edge_ctx.tracker.peek();
                if would_create_cycle(&tracker, &edge) {
                    debug!("Rejecting edge {:?} -> {:?}: would create cycle", from, to);
                    return;
                }
                if matches!(to_node.kind, ui::NodeKind::Leaf { .. }) && tracker.has_parent(to) {
                    debug!(
                        "Rejecting edge {:?} -> {:?}: leaf nodes can only have one parent",
                        from, to
                    );
                    return;
                }
            }

            // root can only have one child
            if let ui::NodeKind::Root = from_node.kind {
                edge_ctx
                    .tracker
                    .with_mut(|tracker| tracker.remove_first_edge_from(&edge));
            }
            edge_ctx.tracker.with_mut(|tracker| tracker.insert(edge));
        }
    };
    node::InputPortHandlers::new(on_mouse_up)
}

fn output_port_handlers(mut temp_edge_ctx: edge::temporary::Context) -> node::OutputPortHandlers {
    let on_mouse_down = move |indexed_drag_offset: IndexedDragOffset| {
        let offset = indexed_drag_offset.offset;
        temp_edge_ctx.set_dragged_from(indexed_drag_offset.id);
        temp_edge_ctx.update_edge(PointEdge {
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

fn context_menu_handlers(
    mut ui_nodes: Signal<ui::NodeMap>,
    mut ctx_menu_state: Signal<node::ContextMenuState>,
    mut edge_ctx: edge::Context,
    mut channel_ctx: channel::Context,
) -> node::ContextMenuHandlers {
    let on_delete = move |node_id: NodeId| {
        ui_nodes.with_mut(|elements| {
            elements.remove(&node_id);
        });
        edge_ctx.tracker.with_mut(|tracker| {
            tracker.remove_node(node_id);
        });
        channel_ctx.tracker.with_mut(|tracker| {
            tracker.remove_node(node_id);
        })
    };

    let on_close = move |_| {
        ctx_menu_state.with_mut(|state| state.is_visible = false);
    };
    node::ContextMenuHandlers::new(on_delete, on_close)
}

fn edge_context_menu_handlers(
    mut edge_ctx_menu_state: Signal<edge::ContextMenuState>,
    mut edge_ctx: edge::Context,
) -> edge::ContextMenuHandlers {
    let on_delete = move |edge_index: usize| {
        edge_ctx.tracker.with_mut(|tracker| {
            tracker.remove_edge(edge_index);
        });
        edge_ctx_menu_state.with_mut(|state| state.is_visible = false);
    };

    let on_close = move |_| {
        edge_ctx_menu_state.with_mut(|state| state.is_visible = false);
    };

    edge::ContextMenuHandlers::new(on_delete, on_close)
}

fn sender_handlers(
    mut channel_temp_connection_ctx: channel::temporary::Context,
) -> SenderPortHandlers {
    let on_mouse_down = move |(origin, indexed_drag_offset, msg_hash): (
        ConnectionOrigin,
        IndexedDragOffset,
        MessageHash,
    )| {
        let offset = indexed_drag_offset.offset;
        channel_temp_connection_ctx.set_dragged(origin, indexed_drag_offset.id, msg_hash);
        channel_temp_connection_ctx.update_edge(PointEdge {
            start: offset,
            end: offset,
        });
    };
    SenderPortHandlers::new(on_mouse_down)
}

fn receiver_handlers(
    mut channel_temp_connection_ctx: channel::temporary::Context,
    mut ui_nodes: Signal<ui::NodeMap>,
) -> ReceiverPortHandlers {
    let on_mouse_down = move |(origin, indexed_drag_offset, msg_hash): (
        ConnectionOrigin,
        IndexedDragOffset,
        MessageHash,
    )| {
        let offset = indexed_drag_offset.offset;
        channel_temp_connection_ctx.set_dragged(origin, indexed_drag_offset.id, msg_hash);
        channel_temp_connection_ctx.update_edge(PointEdge {
            start: offset,
            end: offset,
        });
    };

    let on_context_menu = move |(node_id, message_hash): (NodeId, MessageHash)| {
        ui_nodes.with_mut(|nodes| {
            if let Some(node) = nodes.get_mut(&node_id)
                && let ui::NodeKind::Leaf {
                    external_receivers, ..
                } = &mut node.kind
            {
                if external_receivers.contains(&message_hash) {
                    external_receivers.remove(&message_hash);
                } else {
                    external_receivers.insert(message_hash);
                }
            }
        });
    };

    ReceiverPortHandlers::new(on_mouse_down, on_context_menu)
}

fn channel_handlers(
    mut drag_channel_state: Signal<DragChannelState>,
    mut channel_temp_connection_ctx: channel::temporary::Context,
    mut channel_tracker: Signal<channel::Tracker>,
) -> channel::Handlers {
    let on_drag_start = move |(id, offset): (ChannelId, Point)| {
        drag_channel_state.set(DragChannelState::Dragged { id, offset });
    };

    let receiver_on_mouse_up = move |(id, channel_msg_hash): (ChannelId, MessageHash)| {
        if let Some((origin, from, receiver_msg_hash)) = channel_temp_connection_ctx.take_dragged()
            && matches!(origin, ConnectionOrigin::Receiver)
            && channel_msg_hash == receiver_msg_hash
        {
            channel_tracker.with_mut(|tracker| tracker.connect_receiver(from, id));
        }
    };

    let sender_on_mouse_up = move |(id, channel_msg_hash): (ChannelId, MessageHash)| {
        if let Some((origin, from, sender_msg_hash)) = channel_temp_connection_ctx.take_dragged()
            && matches!(origin, ConnectionOrigin::Sender)
            && channel_msg_hash == sender_msg_hash
        {
            channel_tracker.with_mut(|tracker| tracker.connect_sender(from, id));
        }
    };

    channel::Handlers::new(on_drag_start, receiver_on_mouse_up, sender_on_mouse_up)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::edge;

    #[test]
    fn test_cycle_detection() {
        let mut tracker = edge::Tracker::new();
        tracker.insert(NodeEdge { from: 1, to: 2 });
        tracker.insert(NodeEdge { from: 2, to: 3 });
        // Adding 3 -> 1 would create a cycle
        let cycle_edge = NodeEdge { from: 3, to: 1 };
        assert!(would_create_cycle(&tracker, &cycle_edge));
        // Adding 1 -> 4 would not create a cycle
        let safe_edge = NodeEdge { from: 1, to: 4 };
        assert!(!would_create_cycle(&tracker, &safe_edge));
    }

    #[test]
    fn test_has_parent_detection() {
        let mut tracker = edge::Tracker::new();
        tracker.insert(NodeEdge { from: 1, to: 2 });
        // Node 2 should have a parent
        assert!(tracker.has_parent(2));
        // Node 1 should not have a parent
        assert!(!tracker.has_parent(1));
        // Non-existent node should not have a parent
        assert!(!tracker.has_parent(999));
    }

    #[test]
    fn test_path_detection() {
        let mut tracker = edge::Tracker::new();
        // Create a path: 1 -> 2 -> 3 -> 4
        tracker.insert(NodeEdge { from: 1, to: 2 });
        tracker.insert(NodeEdge { from: 2, to: 3 });
        tracker.insert(NodeEdge { from: 3, to: 4 });
        // Should find path from 1 to 4
        assert!(tracker.has_path(1, 4));
        // Should find path from 2 to 4
        assert!(tracker.has_path(2, 4));
        // Should not find path from 4 to 1
        assert!(!tracker.has_path(4, 1));
        // Should not find path to non-existent node
        assert!(!tracker.has_path(1, 999));
    }
}

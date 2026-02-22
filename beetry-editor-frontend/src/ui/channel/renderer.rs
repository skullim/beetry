use crate::Point;
use beetry_editor_backend::EditorService;
use beetry_editor_backend::api;
use beetry_editor_backend::api::{ChannelQueryView, ChannelUiQueryApi, NodeUiQueryApi};
use beetry_editor_types::id::ChannelId;
use beetry_editor_types::{id::NodeId, spec::node::NodePortKind};
use beetry_plugin::Named;
use dioxus::prelude::*;

use crate::Backend;
use crate::definitions::EdgePos;
use crate::signals::{RequestChannelEdgeRender, RequestChannelRender};
use crate::ui::channel::Channel;
use crate::ui::channel::edge::Edge;
use crate::ui::channel::edge_menu::ConnectionId;
use crate::ui::channel::layout as channel_layout;
use crate::ui::error::ErrorQueueState;
use crate::ui::node::base::{NODE_HEIGHT, NODE_WIDTH};
use crate::ui::node::port::layout;
use crate::ui::text;
use beetry_editor_backend::api::SpecByNodeIdQueryView;
use beetry_editor_backend::api::node::ports::RowIndex;

struct ConnectionEntry {
    channel_id: ChannelId,
    node_port_center: Point,
    channel_pos: Point,
    connection: ConnectionId,
}

// Conditions to re-render channel elements:
// - new channel created
// - channel position updated
#[component]
pub fn Renderer(render_channels: RequestChannelRender) -> Element {
    debug!("rendering");
    render_channels.track();

    let backend = use_context::<Backend>();
    let read = backend.read();
    let query_api = api::ui::channel::borrow(&(*read));

    let channels = query_api.iter().map(|(id, data)| {
        rsx! {
            Channel { key: "{id}", id: *id, position: data.position }
        }
    });

    rsx! {
        {channels}
    }
}

// Conditions to re-render channel connections:
// - node/channel position updated
// - channel connections changed
#[component]
pub fn ConnectionRenderer(render_channel_edges: RequestChannelEdgeRender) -> Element {
    debug!("rendering connection");
    render_channel_edges.track();

    let backend = use_context::<Backend>();
    let backend_peek = backend.peek();
    let editor = &*backend_peek;
    let channel_query_api = api::ui::channel::borrow(editor);
    let channel_data_query_api = api::channel::borrow(editor);
    let node_query_api = api::ui::node::borrow(editor);
    let spec_query_api = api::node::spec::by_node_id(editor);
    let errors = use_context::<ErrorQueueState>();

    let render_edge = |pos: EdgePos, connection: ConnectionId, stroke: &'static str| {
        rsx! {
            Edge {
                key: "{connection.node_id}:{connection.port_id}:{connection.channel_id}",
                pos,
                connection,
                stroke,
            }
        }
    };

    let sender_connections = connection_entries(
        editor,
        NodePortKind::Sender,
        errors,
        &channel_query_api,
        &node_query_api,
        &spec_query_api,
    )
    .map(|entry| {
        render_edge(
            sender_edge_pos(entry.node_port_center, &entry.channel_pos),
            entry.connection,
            "#10B981",
        )
    });

    let receiver_connections = connection_entries(
        editor,
        NodePortKind::Receiver,
        errors,
        &channel_query_api,
        &node_query_api,
        &spec_query_api,
    )
    .filter_map(|entry| {
        let channel_name = channel_data_query_api.spec(entry.channel_id).ok()?.name();
        let channel_body_width = text::text_width_from(channel_name, text::FONT_SIZE_NORMAL);
        Some(render_edge(
            receiver_edge_pos(
                entry.node_port_center,
                &entry.channel_pos,
                channel_body_width,
            ),
            entry.connection,
            "#6B7280",
        ))
    });

    rsx! {
        {sender_connections}
        {receiver_connections}
    }
}

fn connection_entries<'a, CQ, NQ, SQ>(
    editor: &'a EditorService,
    kind: NodePortKind,
    mut errors: ErrorQueueState,
    channel_query_api: &'a CQ,
    node_query_api: &'a NQ,
    spec_query_api: &'a SQ,
) -> impl Iterator<Item = ConnectionEntry>
where
    CQ: ChannelUiQueryApi,
    NQ: NodeUiQueryApi,
    SQ: SpecByNodeIdQueryView,
{
    api::node::ports::connection_views_by_kind(editor, kind)
        .filter_map(move |conn| {
            conn.map_err(|e| {
                errors.push(e);
            })
            .ok()
        })
        .filter_map(move |conn| {
            let connection = ConnectionId {
                node_id: conn.node_id,
                port_id: conn.port_id,
                channel_id: conn.channel_id,
            };
            resolve_connection_entry(
                editor,
                node_query_api,
                channel_query_api,
                spec_query_api,
                connection,
                conn.msg_desc,
                kind,
            )
        })
}

fn resolve_connection_entry(
    editor: &EditorService,
    node_query_api: &impl NodeUiQueryApi,
    channel_query_api: &impl ChannelUiQueryApi,
    spec_query_api: &impl SpecByNodeIdQueryView,
    connection: ConnectionId,
    msg_desc: &str,
    kind: NodePortKind,
) -> Option<ConnectionEntry> {
    let node_pos = node_query_api.position(connection.node_id).ok()?;
    let channel_pos = channel_query_api.position(connection.channel_id).ok()?;
    let port_width = text::text_width_from(msg_desc, text::FONT_SIZE_NORMAL);
    let row_idx: RowIndex =
        api::node::ports::port_order(editor, kind, connection.node_id, connection.port_id).ok()?;
    let node_port_center = node_port_center(
        spec_query_api,
        connection.node_id,
        kind,
        row_idx,
        node_pos,
        port_width,
    )?;
    Some(ConnectionEntry {
        channel_id: connection.channel_id,
        node_port_center,
        channel_pos: *channel_pos,
        connection,
    })
}

fn sender_edge_pos(node_port_center: Point, channel_pos: &Point) -> EdgePos {
    EdgePos {
        start: node_port_center,
        end: Point {
            x: channel_pos.x + channel_layout::PORT_CENTER.x,
            y: channel_pos.y + channel_layout::PORT_CENTER.y,
        },
    }
}

fn receiver_edge_pos(
    node_port_center: Point,
    channel_pos: &Point,
    channel_body_width: f64,
) -> EdgePos {
    EdgePos {
        start: node_port_center,
        end: Point {
            x: channel_pos.x
                + channel_layout::PORT_WIDTH
                + channel_body_width
                + channel_layout::PORT_CENTER.x,
            y: channel_pos.y + channel_layout::PORT_CENTER.y,
        },
    }
}

fn node_port_center(
    spec_query_api: &impl SpecByNodeIdQueryView,
    node_id: NodeId,
    kind: NodePortKind,
    row_idx: RowIndex,
    node_pos: &Point,
    port_width: f64,
) -> Option<Point> {
    let ports = spec_query_api.ports(node_id).ok()?;
    let port_count = match kind {
        NodePortKind::Sender => ports.sender_ids().count(),
        NodePortKind::Receiver => ports.receiver_ids().count(),
    };

    let step = port_step_for(NODE_HEIGHT, port_count);
    let y = node_pos.y + row_idx as f64 * step + (layout::HEIGHT / 2.0);

    let x = match kind {
        NodePortKind::Sender => node_pos.x + NODE_WIDTH + (port_width / 2.0),
        NodePortKind::Receiver => node_pos.x - (port_width / 2.0),
    };

    Some(Point { x, y })
}

fn port_step_for(height: f64, count: usize) -> f64 {
    let intervals = count.saturating_sub(1);
    if intervals == 0 {
        0.0
    } else {
        ((height - layout::HEIGHT) / intervals as f64).max(layout::HEIGHT + layout::MIN_GAP)
    }
}

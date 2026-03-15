use beetry_editor_backend::{
    EditorService, api,
    api::{
        ChannelQueryView, ChannelUiQuery, NodeUiQuery, SpecByNodeIdQuery, node::ports::RowIndex,
    },
    node::PortConnectionQuery,
};
use beetry_editor_types::{
    id::{ChannelId, NodeId, PortConnectionId},
    spec::node::NodePortKind,
};
use beetry_plugin::Named;
use dioxus::prelude::*;

use crate::{
    Backend, Point,
    definitions::EdgePos,
    signals::{RequestChannelEdgeRender, RequestChannelRender},
    ui::{
        channel::{self, Channel, edge::Edge},
        error::ErrorQueueState,
        node::{
            base::{NODE_HEIGHT, NODE_WIDTH},
            port::{ConnectionOrigin, layout},
        },
        text,
    },
};

struct ConnectionEntry {
    channel_id: ChannelId,
    node_port_center: Point,
    port_width: f64,
    channel_pos: Point,
    conn: PortConnectionId,
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
    let query_api = api::ui::channel::query(&(*read));

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
    let channel_query_api = api::ui::channel::query(editor);
    let channel_data_query_api = api::channel::query(editor);
    let node_query_api = api::ui::node::query(editor);
    let spec_query_api = api::node::spec::by_node_id(editor);
    let port_conn_query_api = api::node::ports::connections_query(editor);
    let errors = use_context::<ErrorQueueState>();

    let render_edge = |pos: EdgePos,
                       port_center: Point,
                       conn: PortConnectionId,
                       stroke: &'static str,
                       origin: ConnectionOrigin,
                       port_width: f64| {
        rsx! {
            Edge {
                key: "{conn.node_id}:{conn.port_id}:{conn.channel_id}",
                start: pos.start,
                end: pos.end,
                port_center,
                conn,
                stroke,
                origin,
                port_width,
            }
        }
    };

    let sender_connections = connection_entries(
        editor,
        NodePortKind::Sender,
        errors,
        &port_conn_query_api,
        &channel_query_api,
        &node_query_api,
        &spec_query_api,
    )
    .map(|entry| {
        let pos = sender_edge_pos(entry.node_port_center, &entry.channel_pos);
        render_edge(
            pos,
            entry.node_port_center,
            entry.conn,
            "#10B981",
            ConnectionOrigin::Sender,
            entry.port_width,
        )
    });

    let receiver_connections = connection_entries(
        editor,
        NodePortKind::Receiver,
        errors,
        &port_conn_query_api,
        &channel_query_api,
        &node_query_api,
        &spec_query_api,
    )
    .filter_map(|entry| {
        let channel_name = channel_data_query_api.spec(entry.channel_id).ok()?.name();
        let channel_body_width = text::text_width_from(channel_name, text::FONT_SIZE_NORMAL);
        let pos = receiver_edge_pos(
            entry.node_port_center,
            &entry.channel_pos,
            channel_body_width,
        );
        Some(render_edge(
            pos,
            entry.node_port_center,
            entry.conn,
            "#6B7280",
            ConnectionOrigin::Receiver,
            entry.port_width,
        ))
    });

    rsx! {
        {sender_connections}
        {receiver_connections}
    }
}

fn connection_entries<'a, CQ, NQ, SQ, PQ>(
    editor: &'a EditorService,
    kind: NodePortKind,
    mut errors: ErrorQueueState,
    port_conn_query_api: &'a PQ,
    channel_query_api: &'a CQ,
    node_query_api: &'a NQ,
    spec_query_api: &'a SQ,
) -> impl Iterator<Item = ConnectionEntry> + 'a
where
    CQ: ChannelUiQuery,
    NQ: NodeUiQuery,
    SQ: SpecByNodeIdQuery,
    PQ: PortConnectionQuery,
{
    port_conn_query_api
        .all_connections()
        .filter_map(move |conn| {
            let port_spec = spec_query_api
                .ports(conn.node_id)
                .and_then(|ports| ports.spec(conn.port_id))
                .map_err(|e| {
                    errors.push(e);
                })
                .ok()?;

            if port_spec.kind != kind {
                return None;
            }

            resolve_connection_entry(
                editor,
                node_query_api,
                channel_query_api,
                spec_query_api,
                conn,
                port_spec.msg_spec.as_str(),
                kind,
            )
        })
}

fn resolve_connection_entry(
    editor: &EditorService,
    node_query_api: &impl NodeUiQuery,
    channel_query_api: &impl ChannelUiQuery,
    spec_query_api: &impl SpecByNodeIdQuery,
    conn: PortConnectionId,
    msg_desc: &str,
    kind: NodePortKind,
) -> Option<ConnectionEntry> {
    let node_pos = node_query_api.position(conn.node_id).ok()?;
    let channel_pos = channel_query_api.position(conn.channel_id).ok()?;
    let port_width = text::text_width_from(msg_desc, text::FONT_SIZE_NORMAL);
    let row_idx: RowIndex =
        api::node::ports::order(editor, kind, conn.node_id, conn.port_id).ok()?;
    let node_port_center = node_port_center(
        spec_query_api,
        conn.node_id,
        kind,
        row_idx,
        node_pos,
        port_width,
    )?;
    Some(ConnectionEntry {
        channel_id: conn.channel_id,
        node_port_center,
        port_width,
        channel_pos: *channel_pos,
        conn,
    })
}

fn sender_edge_pos(node_port_center: Point, channel_pos: &Point) -> EdgePos {
    EdgePos {
        start: node_port_center,
        end: Point {
            x: channel_pos.x + channel::layout::PORT_CENTER.x,
            y: channel_pos.y + channel::layout::PORT_CENTER.y,
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
                + channel::layout::PORT_WIDTH
                + channel_body_width
                + channel::layout::PORT_CENTER.x,
            y: channel_pos.y + channel::layout::PORT_CENTER.y,
        },
    }
}

fn node_port_center(
    spec_query_api: &impl SpecByNodeIdQuery,
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
    #[expect(
        clippy::cast_precision_loss,
        reason = "row index is reasonably small number"
    )]
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
        #[expect(
            clippy::cast_precision_loss,
            reason = "intervals is reasonably small number"
        )]
        ((height - layout::HEIGHT) / intervals as f64).max(layout::HEIGHT + layout::MIN_GAP)
    }
}

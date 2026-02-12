use crate::Point;
use beetry_editor_backend::api::{ChannelUiQueryApi, NodeUiQueryApi};
use beetry_editor_types::{id::NodePortId, spec::node::NodePortKind};
use dioxus::prelude::*;

use crate::definitions::EdgePos;
use crate::editor::ServiceContext;
use crate::signals::{RequestChannelEdgeRender, RequestChannelRender};
use crate::ui::channel::{Channel, ReceiverConnection, SenderConnection};
use crate::ui::error_dialog::ErrorMsgQueueSignal;
use crate::ui::text;

// Conditions to re-render channel elements:
// - new channel created
// - channel position updated
#[component]
pub fn Renderer(render_channels: RequestChannelRender) -> Element {
    debug!("rendering");
    render_channels.track();

    let service = use_context::<ServiceContext>();
    let read = service.read();
    let query_api = beetry_editor_backend::api::ui::channel::borrow(&(*read));

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
    debug!("rendering");
    render_channel_edges.track();

    let service = use_context::<ServiceContext>();
    let mut error_queue = use_context::<ErrorMsgQueueSignal>();
    let read = service.read();
    let channel_query_api = beetry_editor_backend::api::ui::channel::borrow(&(*read));
    let node_query_api = beetry_editor_backend::api::ui::node::borrow(&(*read));

    let sender_connections = beetry_editor_backend::api::node::ports::connection_views_by_kind(
        &(*read),
        NodePortKind::Sender,
    )
    .filter_map(move |conn| match conn {
        Ok(conn) => Some(conn),
        Err(err) => {
            error_queue.with_mut(|q| q.push("channel-render", err.to_string()));
            None
        }
    })
    .filter_map(|conn| {
        let node_pos = node_query_api.position(conn.node_id).ok()?;
        let channel_pos = channel_query_api.position(conn.channel_id).ok()?;
        Some(sender_edge_pos(
            node_pos,
            channel_pos,
            conn.msg_desc,
            conn.port_id,
        ))
    })
    .map(|edge| {
        rsx! {
            SenderConnection { edge }
        }
    });

    let receiver_connections = beetry_editor_backend::api::node::ports::connection_views_by_kind(
        &(*read),
        NodePortKind::Receiver,
    )
    .filter_map(move |conn| match conn {
        Ok(conn) => Some(conn),
        Err(err) => {
            error_queue.with_mut(|q| q.push("channel-render", err.to_string()));
            None
        }
    })
    .filter_map(|conn| {
        let node_pos = node_query_api.position(conn.node_id).ok()?;
        let channel_pos = channel_query_api.position(conn.channel_id).ok()?;
        Some(receiver_edge_pos(
            node_pos,
            channel_pos,
            conn.msg_desc,
            conn.port_id,
        ))
    })
    .map(|edge| {
        rsx! {
            ReceiverConnection { edge }
        }
    });

    rsx! {
        {sender_connections}
        {receiver_connections}
    }
}

fn sender_edge_pos(
    node_pos: &Point,
    channel_pos: &Point,
    msg_desc: &str,
    port_id: NodePortId,
) -> EdgePos {
    let port_width = text::text_width_from(msg_desc, 11);

    EdgePos {
        start: Point {
            x: node_pos.x + 100.0 + port_width,
            // @todo port_id should be changed here
            y: node_pos.y + 10.0 + 10.0 + 20.0 * port_id.raw_value() as f64,
        },
        end: Point {
            x: channel_pos.x + 20.0,
            y: channel_pos.y + 12.0,
        },
    }
}

fn receiver_edge_pos(
    node_pos: &Point,
    channel_pos: &Point,
    msg_desc: &str,
    port_id: NodePortId,
) -> EdgePos {
    let port_width = text::text_width_from(msg_desc, 11);

    EdgePos {
        start: Point {
            x: node_pos.x,
            y: node_pos.y + 10.0 + 10.0 + 20.0 * port_id.raw_value() as f64, // Middle of port vertically
        },
        end: Point {
            x: channel_pos.x + 20.0 + port_width + 20.0, // Offset to center of receiver port dot
            y: channel_pos.y + 12.0,                     // Offset to center of channel vertically
        },
    }
}

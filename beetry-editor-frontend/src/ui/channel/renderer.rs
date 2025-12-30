use crate::Point;
use beetry_editor_types::id::{ChannelId, NodeId, NodePortId};
use beetry_editor_types::output::node::PortConnectionState;
use beetry_editor_types::spec::node::NodePortKind;
use dioxus::prelude::*;

use crate::definitions::EdgePos;
use crate::editor::ServiceContext;
use crate::signals::RequestRender;
use crate::ui::channel::{Channel, ReceiverConnection, SenderConnection};
use crate::ui::text;

// Conditions to re-render the channels:
// - new channel created
// - node/channel position updated
#[component]
pub fn Renderer(render_channels: Signal<RequestRender>) -> Element {
    debug!("rendering channels");
    let _read = render_channels.read();

    let service = use_context::<ServiceContext>();
    let read = service.service.read();
    let ui_api = read.ui_api();
    let channel_api = ui_api.channel();
    let channels = channel_api.iter().map(|(id, data)| {
        rsx! {
            Channel { key: "{id}", id: *id, position: data.position }
        }
    });

    let port_channel_conns = port_channel_conns(&service);
    let node_api = read.node_api();
    let sender_conns = port_channel_conns.iter().filter(|conn| {
        node_api
            .spec()
            .ports(conn.node_id)
            .ok()
            .is_some_and(|ports_spec| {
                ports_spec
                    .kind(conn.port_id)
                    .is_some_and(|kind| kind == NodePortKind::Sender)
            })
    });

    let sender_connections = sender_conns
        .filter_map(|conn| sender_edge_pos(&service, conn).ok())
        .map(|edge| {
            rsx! {
                SenderConnection { edge }
            }
        });

    let receiver_conns = port_channel_conns.iter().filter(|conn| {
        node_api
            .spec()
            .ports(conn.node_id)
            .ok()
            .is_some_and(|ports_spec| {
                ports_spec
                    .kind(conn.port_id)
                    .is_some_and(|kind| kind == NodePortKind::Receiver)
            })
    });

    let receiver_connections = receiver_conns
        .filter_map(|conn| receiver_edge_pos(&service, conn).ok())
        .map(|edge| {
            rsx! {
                ReceiverConnection { edge }
            }
        });

    rsx! {
        {sender_connections}
        {receiver_connections}
        {channels}
    }
}

struct PortChannelConnection {
    node_id: NodeId,
    port_id: NodePortId,
    channel_id: ChannelId,
}

fn sender_edge_pos(service: &ServiceContext, conn: &PortChannelConnection) -> Result<EdgePos> {
    let read = service.read();
    let ui_api = read.ui_api();
    let ui_api_node = ui_api.node();
    let ui_api_channel = ui_api.channel();

    let node_pos = ui_api_node.position(conn.node_id)?;
    let channel_pos = ui_api_channel.position(conn.channel_id)?;

    let node_api = read.node_api();
    let spec_api = node_api.spec();
    let spec = spec_api.spec_by_node_id_pub(conn.node_id)?;
    let ports_spec = spec.ports().as_ref().unwrap();
    let msg_desc = ports_spec.spec(conn.port_id).unwrap().msg_spec.as_str();
    let port_width = text::text_width_from(msg_desc, 11);

    Ok(EdgePos {
        start: Point {
            x: node_pos.x + 100.0 + port_width,
            // @todo port_id should be changed here
            y: node_pos.y + 10.0 + 10.0 + 20.0 * conn.port_id.raw_value() as f64,
        },
        end: Point {
            x: channel_pos.x + 20.0,
            y: channel_pos.y + 12.0,
        },
    })
}

fn receiver_edge_pos(service: &ServiceContext, conn: &PortChannelConnection) -> Result<EdgePos> {
    let read = service.read();
    let ui_api = read.ui_api();
    let ui_api_node = ui_api.node();
    let ui_api_channel = ui_api.channel();

    let node_pos = ui_api_node.position(conn.node_id)?;
    let channel_pos = ui_api_channel.position(conn.channel_id)?;

    let node_api = read.node_api();
    let spec_api = node_api.spec();
    let spec = spec_api.spec_by_node_id_pub(conn.node_id)?;
    let ports_spec = spec.ports().as_ref().unwrap();
    let msg_desc = ports_spec.spec(conn.port_id).unwrap().msg_spec.as_str();

    let port_width = text::text_width_from(msg_desc, 11);

    Ok(EdgePos {
        start: Point {
            x: node_pos.x,
            y: node_pos.y + 10.0 + 10.0 + 20.0 * conn.port_id.raw_value() as f64, // Middle of port vertically
        },
        end: Point {
            x: channel_pos.x + 20.0 + port_width + 20.0, // Offset to center of receiver port dot
            y: channel_pos.y + 12.0,                     // Offset to center of channel vertically
        },
    })
}

//@todo would be nice to do without allocations
fn port_channel_conns(service: &ServiceContext) -> Vec<PortChannelConnection> {
    let read = service.read();
    let node_api = read.node_api();
    let port_state = node_api.port_state();
    port_state
        .iter()
        .flat_map(|(id, port_conns_iter)| {
            port_conns_iter.flat_map(|(port_id, state)| match state {
                PortConnectionState::Internal(conns) => conns
                    .iter()
                    .map(|channel_id| PortChannelConnection {
                        node_id: *id,
                        port_id: *port_id,
                        channel_id: *channel_id,
                    })
                    .collect::<Vec<_>>(),
                PortConnectionState::External => Vec::new(),
            })
        })
        .collect::<Vec<_>>()
}

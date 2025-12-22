use dioxus::prelude::*;

use crate::definitions::{EdgePos, Point};
use crate::editor::ServiceContext;
use crate::ui::channel::tracker::Tracker;
use crate::ui::channel::{Channel, Channel2, ReceiverConnection, SenderConnection};
use crate::ui::{self, text};

// Conditions to re-render the channels:
// - new channel created
// - node/channel position updated
#[component]
pub fn Renderer(tracker: ReadSignal<Tracker>, ui_nodes: ReadSignal<ui::NodeMap>) -> Element {
    let nodes_read = ui_nodes.read();
    let tracker_read = tracker.read();

    let read_channels = tracker_read.channels();
    let channels = read_channels.iter().map(|(id, element)| {
        rsx! {
            Channel {
                pos: element.pos,
                id: *id,
                spec: element.snapshot.spec().clone(),
            }
        }
    });

    let read_senders = tracker_read.senders();
    let sender_connections = read_senders.iter().flat_map(|(node_id, channels)| {
        channels.iter().enumerate().map(|(idx, channel_id)| {
            let channel = tracker_read.channel(*channel_id).unwrap();
            let node = nodes_read.get(node_id).unwrap();

            let port_width = text::text_width_from(channel.snapshot.spec().as_str(), 11);
            let start = Point {
                x: node.pos.x + 100.0 + port_width,
                y: node.pos.y + 10.0 + 10.0 + 20.0 * idx as f64, // Middle of port vertically
            };

            let mut end = channel.pos;
            end.x += 20.0; // Offset to center of sender port dot
            end.y += 12.0; // Offset to center of channel vertically

            let edge = EdgePos { start, end };
            rsx! {
                SenderConnection { edge }
            }
        })
    });

    let read_receivers = tracker_read.receivers();
    let receiver_connections = read_receivers.iter().flat_map(|(node_id, channels)| {
        channels.iter().enumerate().map(|(idx, channel_id)| {
            let channel = tracker_read.channel(*channel_id).unwrap();
            let node = nodes_read.get(node_id).unwrap();

            let start = Point {
                x: node.pos.x,
                y: node.pos.y + 10.0 + 10.0 + 20.0 * idx as f64, // Middle of port vertically
            };
            let body_width = text::text_width_from(channel.snapshot.spec().as_str(), 11);

            let mut end = channel.pos;
            end.x += 38.0 + body_width + 20.0; // Offset to center of receiver port dot
            end.y += 12.0; // Offset to center of channel vertically

            let edge = EdgePos { start, end };
            rsx! {
                ReceiverConnection { edge }
            }
        })
    });

    rsx! {
        {sender_connections}
        {receiver_connections}
        {channels}
    }
}

#[component]
pub fn Renderer2() -> Element {
    let service = use_context::<ServiceContext>();
    let read = service.service.read();
    let ui_api = read.ui_api();
    let channel_api = ui_api.channel();
    let channels = channel_api.iter().map(|(id, _)| {
        rsx! {
            Channel2 { id: *id }
        }
    });

    // let node_api = read.node_api();
    // let tracker = node_api.tracker();
    // let sender_connections = tracker.nodes().flat_map(|id| {
    //     let port_state_api = node_api.port_state();
    //     let port_iter = port_state_api.port_iter(*id);

    //     // @todo get real port spec
    //     let spec_as_str = "port spec";
    //     let port_width = text::text_width_from(spec_as_str, 11);

    //     port_iter.flat_map(|(port_id, port_state)| {
    //         let ui_api_node = ui_api.node();
    //         let node_pos = ui_api_node.position(*id).unwrap();
    //         let start = Point {
    //             x: node_pos.origin.x + 100.0 + port_width,
    //             // @todo port_id should be changed here
    //             y: node_pos.origin.y + 10.0 + 10.0 + 20.0 * (*port_id) as f64,
    //         };

    //         port_state.connected().flat_map(|channel_id| {
    //             let channel_pos = ui_api.channel().position(*channel_id).unwrap().origin;

    //             let end = Point {
    //                 x: channel_pos.x + 20.0,
    //                 y: channel_pos.y + 12.0,
    //             };

    //             let edge = EdgePos { start, end };

    //             rsx! {
    //                 SenderConnection { edge }
    //             }
    //         })
    //     })
    // });

    // let read_receivers = tracker_read.receivers();
    // let receiver_connections = read_receivers.iter().flat_map(|(node_id, channels)| {
    //     channels.iter().enumerate().map(|(idx, channel_id)| {
    //         let channel = tracker_read.channel(*channel_id).unwrap();
    //         let node = nodes_read.get(node_id).unwrap();

    //         let start = Point {
    //             x: node.pos.x,
    //             y: node.pos.y + 10.0 + 10.0 + 20.0 * idx as f64, // Middle of port vertically
    //         };
    //         let body_width = text::text_width_from(channel.snapshot.spec().as_str(), 11);

    //         let mut end = channel.pos;
    //         end.x += 38.0 + body_width + 20.0; // Offset to center of receiver port dot
    //         end.y += 12.0; // Offset to center of channel vertically

    //         let edge = EdgePos { start, end };
    //         rsx! {
    //             ReceiverConnection { edge }
    //         }
    //     })
    // });

    rsx! {
        //{sender_connections}
        //{receiver_connections}
        {channels}
    }
}

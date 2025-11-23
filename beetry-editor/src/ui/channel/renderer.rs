use dioxus::prelude::*;

use crate::definitions::{EdgePos, Point};
use crate::ui::channel::tracker::Tracker;
use crate::ui::channel::{Channel, ReceiverConnection, SenderConnection};
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

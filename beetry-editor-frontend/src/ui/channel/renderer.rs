use crate::Point;
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

    //@todo this api calls are messy and somethings are collected to satisfy borrow checker
    let node_api = read.node_api();
    let tracker = node_api.tracker();
    let sender_connections = tracker.nodes().flat_map(|id| {
        let spec_api = node_api.spec();
        let ports = spec_api.ports(*id).unwrap();
        let sender_ids: Vec<_> = ports.sender_ids().copied().collect();
        let port_state_api = node_api.port_state();

        sender_ids
            .into_iter()
            .map(move |port_id| (port_id, port_state_api.state(*id, port_id).unwrap().clone()))
            .flat_map(move |(port_id, port_state)| {
                // @todo get real port spec
                let spec_as_str = "port spec";
                let port_width = text::text_width_from(spec_as_str, 11);

                let read = service.service.read();
                let ui_api = read.ui_api();
                let ui_api_node = ui_api.node();
                let node_pos = ui_api_node.position(*id).unwrap();
                let start = Point {
                    x: node_pos.origin.x + 100.0 + port_width,
                    // @todo port_id should be changed here
                    y: node_pos.origin.y + 10.0 + 10.0 + 20.0 * (port_id.raw_value()) as f64,
                };
                let connected: Vec<_> = port_state.connected().copied().collect();

                connected.into_iter().flat_map(move |channel_id| {
                    let read = service.service.read();
                    let ui_api = read.ui_api();
                    let channel_pos = ui_api.channel().position(channel_id).unwrap().origin;

                    let end = Point {
                        x: channel_pos.x + 20.0,
                        y: channel_pos.y + 12.0,
                    };

                    let edge = EdgePos { start, end };

                    rsx! {
                        SenderConnection { edge }
                    }
                })
            })
    });

    let receiver_connections = tracker.nodes().flat_map(|id| {
        let spec_api = node_api.spec();
        let ports = spec_api.ports(*id).unwrap();
        let receiver_ids: Vec<_> = ports.receiver_ids().copied().collect();
        let port_state_api = node_api.port_state();

        receiver_ids
            .into_iter()
            .map(move |port_id| (port_id, port_state_api.state(*id, port_id).unwrap().clone()))
            .flat_map(move |(port_id, port_state)| {
                // @todo get real port spec
                let spec_as_str = "port spec";
                let port_width = text::text_width_from(spec_as_str, 11);

                let read = service.service.read();
                let ui_api = read.ui_api();
                let ui_api_node = ui_api.node();
                let node_pos = ui_api_node.position(*id).unwrap();
                let start = Point {
                    x: node_pos.origin.x,
                    // @todo port_id should be changed here
                    y: node_pos.origin.y + 10.0 + 10.0 + 20.0 * (port_id.raw_value()) as f64, // Middle of port vertically
                };
                let connected: Vec<_> = port_state.connected().copied().collect();

                connected.into_iter().flat_map(move |channel_id| {
                    let read = service.service.read();
                    let ui_api = read.ui_api();
                    let channel_pos = ui_api.channel().position(channel_id).unwrap().origin;

                    let end = Point {
                        x: channel_pos.x + 20.0 + port_width + 20.0, // Offset to center of receiver port dot
                        y: channel_pos.y + 12.0, // Offset to center of channel vertically
                    };

                    let edge = EdgePos { start, end };

                    rsx! {
                        ReceiverConnection { edge }
                    }
                })
            })
    });

    rsx! {
        {sender_connections}
        {receiver_connections}
        {channels}
    }
}

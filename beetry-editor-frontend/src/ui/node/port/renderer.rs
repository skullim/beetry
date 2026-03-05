use crate::ui::error::ErrorQueueState;
use crate::ui::node::port::{self, ConnectionOrigin, layout};
use crate::{Backend, Point};
use beetry_editor_backend::api;
use beetry_editor_backend::api::SpecByNodeIdQuery;
use beetry_editor_types::id::{NodeId, NodePortId};
use dioxus::prelude::*;
use std::rc::Rc;

#[derive(PartialEq, Clone, Props)]
pub struct RendererProps {
    id: NodeId,
    position: Point,
    dimensions: NodeDimensions,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeDimensions {
    pub width: f64,
    pub height: f64,
}

struct PortsMetadata {
    sender: SenderMetadata,
    receiver: ReceiverMetadata,
}

struct SenderMetadata {
    count: usize,
    port_ids: Vec<NodePortId>,
}

struct ReceiverMetadata {
    count: usize,
    port_ids: Vec<NodePortId>,
}

/// Renders ports for a single leaf node
#[component]
pub(crate) fn Renderer(props: RendererProps) -> Element {
    let backend = use_context::<Backend>();
    let mut errors = use_context::<ErrorQueueState>();

    let id = props.id;
    let Some(ports_meta) = use_hook(|| {
        let read = backend.read();
        let spec_query = api::node::spec::by_node_id(&(*read));
        spec_query
            .spec(id)
            .inspect_err(|err| errors.push(err))
            .ok()
            .and_then(|spec| {
                spec.ports().as_ref().map(|s| {
                    Rc::new(PortsMetadata {
                        sender: SenderMetadata {
                            count: s.sender_ids().count(),
                            port_ids: s.sender_ids().copied().collect(),
                        },
                        receiver: ReceiverMetadata {
                            count: s.receiver_ids().count(),
                            port_ids: s.receiver_ids().copied().collect(),
                        },
                    })
                })
            })
    }) else {
        return rsx!();
    };

    let position = props.position;
    let width = props.dimensions.width;
    let height = props.dimensions.height;

    let receiver_step = step_for(height, ports_meta.receiver.count);
    let sender_step = step_for(height, ports_meta.sender.count);

    let render_port = |port_ids: &[NodePortId], step: f64, x: f64, origin: ConnectionOrigin| {
        rsx! {
            g {
                for (row_idx, port_id) in port_ids.iter().copied().enumerate() {
                    port::Body {
                        key: "{port_id}",
                        id,
                        position: Point {
                            x,
                            y: position.y + row_idx as f64 * step,
                        },
                        port_id,
                        origin,
                    }
                }
            }
        }
    };

    rsx! {
        {render_port(
            &ports_meta.receiver.port_ids,
            receiver_step,
            position.x,
            ConnectionOrigin::Receiver
        )}
        {render_port(
            &ports_meta.sender.port_ids,
            sender_step,
            position.x + width,
            ConnectionOrigin::Sender
        )}
    }
}

fn step_for(height: f64, count: usize) -> f64 {
    let intervals = count.saturating_sub(1);
    if intervals == 0 {
        0.0
    } else {
        ((height - layout::HEIGHT) / intervals as f64).max(layout::HEIGHT + layout::MIN_GAP)
    }
}

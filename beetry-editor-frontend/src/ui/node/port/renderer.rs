use crate::ui::error::ErrorQueueState;
use crate::ui::node::port::{self, ConnectionOrigin, layout};
use crate::{Backend, Point};
use beetry_editor_backend::api::SpecByNodeIdQueryView;
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

struct PortIdRowIdxPair {
    port_id: NodePortId,
    row_idx: usize,
}

fn step_for(height: f64, count: usize) -> f64 {
    let intervals = count.saturating_sub(1);
    if intervals == 0 {
        0.0
    } else {
        ((height - layout::HEIGHT) / intervals as f64).max(layout::HEIGHT + layout::MIN_GAP)
    }
}

#[component]
pub(crate) fn Renderer(props: RendererProps) -> Element {
    let backend = use_context::<Backend>();
    let read = backend.read();
    let spec_query = beetry_editor_backend::api::node::spec::by_node_id(&(*read));
    let errors = use_context::<ErrorQueueState>();

    let id = props.id;
    let height = props.dimensions.height;
    let receiver_step = use_hook(|| {
        step_for(
            height,
            spec_query
                .ports(id)
                .iter()
                .flat_map(|ports_spec| ports_spec.receiver_ids())
                .count(),
        )
    });
    let sender_step = use_hook(|| {
        step_for(
            height,
            spec_query
                .ports(id)
                .iter()
                .flat_map(|ports_spec| ports_spec.sender_ids())
                .count(),
        )
    });

    let mut receiver_errors = errors;
    let receiver_id_pairs: Rc<Vec<_>> = use_hook(|| match spec_query.ports(id) {
        Ok(ports_spec) => Rc::new(
            ports_spec
                .receiver_ids()
                .enumerate()
                .map(|(row_idx, port_id)| PortIdRowIdxPair {
                    port_id: *port_id,
                    row_idx,
                })
                .collect(),
        ),
        Err(e) => {
            receiver_errors.push(e);
            Rc::new(Vec::new())
        }
    });

    let mut sender_errors = errors;
    let sender_id_pairs: Rc<Vec<_>> = use_hook(|| match spec_query.ports(id) {
        Ok(ports_spec) => Rc::new(
            ports_spec
                .sender_ids()
                .enumerate()
                .map(|(row_idx, port_id)| PortIdRowIdxPair {
                    port_id: *port_id,
                    row_idx,
                })
                .collect(),
        ),
        Err(e) => {
            sender_errors.push(e);
            Rc::new(Vec::new())
        }
    });

    let position = props.position;
    let width = props.dimensions.width;

    rsx! {
        g {
            for PortIdRowIdxPair { row_idx, port_id } in receiver_id_pairs.iter() {
                port::Body {
                    key: "{port_id}",
                    id,
                    position: Point {
                        x: position.x,
                        y: position.y + *row_idx as f64 * receiver_step,
                    },
                    port_id: *port_id,
                    origin: ConnectionOrigin::Receiver,
                }
            }
        }
        g {
            for PortIdRowIdxPair { row_idx, port_id } in sender_id_pairs.iter() {
                port::Body {
                    key: "{port_id}",
                    id,
                    position: Point {
                        x: position.x + width,
                        y: position.y + *row_idx as f64 * sender_step,
                    },
                    port_id: *port_id,
                    origin: ConnectionOrigin::Sender,
                }
            }
        }
    }
}

use beetry_editor_backend::api::{NodeTrackerQueryView, NodeUiQueryProcessor};
use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::Backend;
use crate::Point;
use crate::signals::RequestNodeRender;
use crate::ui::error::ErrorQueueState;
use crate::ui::node::control::Control;
use crate::ui::node::leaf::Leaf;
use crate::ui::node::root::Root;
use beetry_editor_types::spec::node::NodeKind;

// Conditions to re-render the nodes:
// - new node created
// - node position updated
#[component]
pub fn Renderer(render_nodes: RequestNodeRender) -> Element {
    debug!("rendering");
    render_nodes.track();

    let backend = use_context::<Backend>();
    let read = backend.read();

    let query = beetry_editor_backend::api::ui::node::borrow(&(*read));
    let query_processor = NodeUiQueryProcessor::new(&query);

    let tracker = beetry_editor_backend::api::node::tracker::query_view(&(*read));
    let mapped_nodes = |kind| {
        let ids = tracker.nodes_by_kind(kind);
        query_processor.map_to_positions(ids)
    };

    let error_queue = use_context::<ErrorQueueState>();

    let controls = mapped_nodes(NodeKind::Control).map(|result| {
        render_node_result(result, error_queue, |id, pos| {
            rsx! {
                Control {
                    key: "{id}",
                    id: *id,
                    position: Point { x: pos.x, y: pos.y },
                }
            }
        })
    });

    let actions = mapped_nodes(NodeKind::action()).map(|result| {
        render_node_result(result, error_queue, |id, pos| {
            rsx! {
                Leaf {
                    key: "{id}",
                    id: *id,
                    position: Point { x: pos.x, y: pos.y },
                }
            }
        })
    });

    let conditions = mapped_nodes(NodeKind::condition()).map(|result| {
        render_node_result(result, error_queue, |id, pos| {
            rsx! {
                Leaf {
                    key: "{id}",
                    id: *id,
                    position: Point { x: pos.x, y: pos.y },
                }
            }
        })
    });

    let root = mapped_nodes(NodeKind::Root).map(|result| {
        render_node_result(result, error_queue, |id, pos| {
            rsx! {
                Root {
                    key: "{id}",
                    id: *id,
                    position: Point { x: pos.x, y: pos.y },
                }
            }
        })
    });

    rsx! {
        {root}
        {actions}
        {conditions}
        {controls}
    }
}

fn render_node_result(
    result: anyhow::Result<(&NodeId, &Point)>,
    mut errors: ErrorQueueState,
    render_ok: impl FnOnce(&NodeId, &Point) -> Element,
) -> Element {
    match result {
        Ok((id, pos)) => render_ok(id, pos),
        Err(e) => {
            errors.push(e);
            rsx!()
        }
    }
}

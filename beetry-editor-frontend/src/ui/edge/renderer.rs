use crate::definitions::EdgePos;
use crate::signals::RequestEdgeRender;
use crate::ui::edge::Edge;
use crate::{Backend, ui::error::ErrorQueueState};
use beetry_editor_backend::api::{EdgeQueryView, NodeUiQueryApi};
use dioxus::prelude::*;

// Conditions to re-render the edges:
// - new edge created
// - node position updated
#[component]
pub fn Renderer(render_edges: RequestEdgeRender) -> Element {
    debug!("rendering");
    render_edges.track();
    let backend = use_context::<Backend>();
    let read = backend.read();

    let edge_query = beetry_editor_backend::api::edge::borrow(&(*read));
    let ui_node_query = beetry_editor_backend::api::ui::node::borrow(&(*read));

    let mut errors = use_context::<ErrorQueueState>();
    let mut node_pos = |node_id| {
        ui_node_query
            .data(node_id)
            //@todo refine on API layer to get position of the port and not node
            .map(|d| d.position)
            .map_err(|e| errors.push(e))
            .ok()
    };

    let edges = edge_query.edges().filter_map(|(id, edge)| {
        let start = node_pos(edge.from)?;
        let end = node_pos(edge.to)?;

        Some(rsx! {
            Edge { key: "{id}", pos: EdgePos { start, end }, edge_id: *id }
        })
    });

    rsx! {
        {edges}
    }
}

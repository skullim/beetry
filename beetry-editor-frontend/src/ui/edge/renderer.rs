use crate::definitions::EdgePos;
use crate::editor::ServiceContext;
use crate::signals::RequestRender;
use crate::ui::edge::Edge;
use beetry_editor_backend::edge::EdgeQueryApi;
use beetry_editor_backend::ui::NodeUiQueryApi;
use dioxus::prelude::*;

// Conditions to re-render the edges:
// - new edge created
// - node position updated
#[component]
pub fn Renderer(render_edges: Signal<RequestRender>) -> Element {
    debug!("rendering");
    let _read = render_edges.read();
    let service = use_context::<ServiceContext>();
    let read = service.read();

    let edge_query = beetry_editor_backend::api::edge::borrow(&(*read));
    let ui_node_query = beetry_editor_backend::api::ui::node::borrow(&(*read));

    let edges = edge_query.edges().map(|(id, edge)| {
        //@todo refine on service layer to get position of the port and not node
        //@todo error handling
        let edge_start = ui_node_query.data(edge.from).unwrap().position;
        let edge_end = ui_node_query.data(edge.to).unwrap().position;
        let edge_pos = EdgePos {
            start: edge_start,
            end: edge_end,
        };
        rsx! {
            Edge { key: "{id}", pos: edge_pos, edge_id: *id }
        }
    });

    rsx! {
        {edges}
    }
}

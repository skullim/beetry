use beetry_editor_backend::ui::NodeUiQueryProcessor;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::Point;
use crate::editor::ServiceContext;
use crate::signals::RequestRender;
use crate::ui::node::control::Control;
use crate::ui::node::leaf::Leaf;
use crate::ui::node::root::Root;
use beetry_editor_types::spec::node::NodeKind;

// Conditions to re-render the nodes:
// - new node created
// - node position updated
#[component]
pub fn Renderer(render_nodes: Signal<RequestRender>) -> Element {
    debug!("rendering");
    let _read = render_nodes.read();

    let service = use_context::<ServiceContext>();
    let read = service.read();
    let query = beetry_editor_backend::api::ui::node::borrow(&(*read));

    let query_processor = NodeUiQueryProcessor::new(&query);
    let node_api = read.node_api();
    let tracker = node_api.tracker();

    let filter_nodes_fn = |kind| {
        query_processor
            .map_to_positions(tracker.nodes_by_kind(kind).copied())
            .filter_map(|r| r.ok())
    };

    let controls = (filter_nodes_fn)(NodeKind::Control).map(|(id, pos)| {
        rsx! {
            Control { key: "{id}", id, position: Point { x: pos.x, y: pos.y } }
        }
    });
    let actions = (filter_nodes_fn)(NodeKind::action()).map(|(id, pos)| {
        rsx! {
            Leaf { key: "{id}", id, position: Point { x: pos.x, y: pos.y } }
        }
    });
    let conditions = (filter_nodes_fn)(NodeKind::condition()).map(|(id, pos)| {
        rsx! {
            Leaf { key: "{id}", id, position: Point { x: pos.x, y: pos.y } }
        }
    });
    let root = (filter_nodes_fn)(NodeKind::Root).map(|(id, pos)| {
        rsx! {
            Root { key: "{id}", id, position: Point { x: pos.x, y: pos.y } }
        }
    });

    rsx! {
        {root}
        {actions}
        {conditions}
        {controls}
    }
}

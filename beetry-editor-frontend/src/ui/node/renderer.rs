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
    let read = service.service.read();
    let ui_api = read.ui_api();
    let ui_node_api = ui_api.node();
    let controls = ui_node_api
        .positions_by_kind(NodeKind::Control)
        .map(|(id, pos)| {
            rsx! {
                Control {
                    key: "{id}",
                    id,
                    position: Point { x: pos.x, y: pos.y },
                }
            }
        });
    let actions = ui_node_api
        .positions_by_kind(NodeKind::action())
        .map(|(id, pos)| {
            rsx! {
                Leaf {
                    key: "{id}",
                    id,
                    position: Point { x: pos.x, y: pos.y },
                }
            }
        });

    let conditions = ui_node_api
        .positions_by_kind(NodeKind::condition())
        .map(|(id, pos)| {
            rsx! {
                Leaf {
                    key: "{id}",
                    id,
                    position: Point { x: pos.x, y: pos.y },
                }
            }
        });

    let root = ui_node_api
        .positions_by_kind(NodeKind::Root)
        .map(|(id, pos)| {
            rsx! {
                Root {
                    key: "{id}",
                    id,
                    position: Point { x: pos.x, y: pos.y },
                }
            }
        });

    rsx! {
        {root}
        {actions}
        {conditions}
        {controls}
    }
}

use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::editor::ServiceContext;
use crate::signals::RequestRender;
use crate::ui::Point;
use crate::ui::node::control::Control;
use crate::ui::node::leaf::Leaf;
use crate::ui::node::root::Root;

// Conditions to re-render the nodes:
// - new node created
// - node position updated
#[component]
pub fn Renderer(render_nodes: Signal<RequestRender>) -> Element {
    debug!("rendering nodes renderer");
    let _read = render_nodes.read();
    let service = use_context::<ServiceContext>();
    let read = service.service.read();
    let ui_api = read.ui_api();
    let ui_node_api = ui_api.node();
    let controls = ui_node_api
        .positions_by_kind(beetry_editor_types::NodeKind::Control)
        .map(|(id, pos)| {
            rsx! {
                Control {
                    key: "{id}",
                    id,
                    position: Point {
                        x: pos.origin.x,
                        y: pos.origin.y,
                    },
                }
            }
        });
    let actions = ui_node_api
        .positions_by_kind(beetry_editor_types::NodeKind::Action)
        .map(|(id, pos)| {
            rsx! {
                Leaf {
                    key: "{id}",
                    id,
                    position: Point {
                        x: pos.origin.x,
                        y: pos.origin.y,
                    },
                }
            }
        });

    let conditions = ui_node_api
        .positions_by_kind(beetry_editor_types::NodeKind::Condition)
        .map(|(id, pos)| {
            rsx! {
                Leaf {
                    key: "{id}",
                    id,
                    position: Point {
                        x: pos.origin.x,
                        y: pos.origin.y,
                    },
                }
            }
        });

    let root = ui_node_api
        .positions_by_kind(beetry_editor_types::NodeKind::Root)
        .map(|(id, pos)| {
            rsx! {
                Root {
                    key: "{id}",
                    id,
                    position: Point {
                        x: pos.origin.x,
                        y: pos.origin.y,
                    },
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

use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::editor::ServiceContext;
use crate::ui::node::control::{Control, Control2};
use crate::ui::node::leaf::Leaf;
use crate::ui::node::root::Root;
use crate::ui::{NodeKind, NodeMap, Point};

// Conditions to re-render the nodes:
// - new node created
// - node position updated
#[component]
pub fn Renderer(ui_nodes: Signal<NodeMap>) -> Element {
    debug!("rendering nodes renderer");
    let rendered_lock = ui_nodes.read();
    let rendered = rendered_lock.iter().map(|(&id, node)| match &node.kind {
        NodeKind::Control { params_schema: _ } => {
            rsx! {
                Control {
                    key: "{id}",
                    id,
                    position: node.pos,
                    name: node.name.clone(),
                }
            }
        }

        NodeKind::Leaf {
            schema,
            external_receivers,
        } => rsx! {
            Leaf {
                key: "{id}",
                id,
                position: node.pos,
                name: node.name.clone(),
                schema: schema.clone(),
                external_receivers: external_receivers.clone(),
            }
        },
        NodeKind::Root => {
            rsx! {
                Root { key: "{id}", id, position: node.pos }
            }
        }
    });

    rsx! {
        {rendered}
    }
}

//@todo add signal for rerendering
#[component]
pub fn Renderer2() -> Element {
    debug!("rendering nodes renderer");
    let service = use_context::<ServiceContext>();
    let read = service.service.read();
    let ui_api = read.ui_api();
    let ui_node_api = ui_api.node();
    let controls = ui_node_api
        .positions_by_kind(beetry_editor_types::NodeKind::Control)
        .map(|(id, pos)| {
            rsx! {
                Control2 {
                    key: "{id}",
                    id,
                    position: Point {
                        x: pos.origin.x,
                        y: pos.origin.y,
                    },
                }
            }
        });
    //@todo implement rendering for remaining node types

    // let rendered_lock = ui_nodes.read();
    // let rendered = rendered_lock.iter().map(|(&id, node)| match &node.kind {
    //     NodeKind::Control { params_schema: _ } => {
    //         rsx! {
    //             Control {
    //                 key: "{id}",
    //                 id,
    //                 position: node.pos,
    //                 name: node.name.clone(),
    //             }
    //         }
    //     }

    //     NodeKind::Leaf {
    //         schema,
    //         external_receivers,
    //     } => rsx! {
    //         Leaf {
    //             key: "{id}",
    //             id,
    //             position: node.pos,
    //             name: node.name.clone(),
    //             schema: schema.clone(),
    //             external_receivers: external_receivers.clone(),
    //         }
    //     },
    //     NodeKind::Root => {
    //         rsx! {
    //             Root { key: "{id}", id, position: node.pos }
    //         }
    //     }
    // });

    rsx! {
        {controls}
    }
}

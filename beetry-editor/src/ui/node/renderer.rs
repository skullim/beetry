use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::ui::node::control::Control;
use crate::ui::node::leaf::Leaf;
use crate::ui::node::root::Root;
use crate::ui::{NodeKind, NodeMap};

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

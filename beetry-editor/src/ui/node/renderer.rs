use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::ui::{
    NodeKind, NodeMap,
    node::{control::Control, leaf::Leaf, root::Root},
};

#[component]
pub(crate) fn Renderer(ui_nodes: Signal<NodeMap>) -> Element {
    debug!("rendering nodes renderer");
    let rendered_lock = ui_nodes.read();
    let rendered = rendered_lock.iter().map(|(&id, node)| match &node.kind {
        NodeKind::Control => {
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

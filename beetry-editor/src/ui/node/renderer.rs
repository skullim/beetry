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
    let rendered = rendered_lock.iter().map(|(&id, data)| match &data.kind {
        NodeKind::Control(kind) => {
            rsx! {
                Control {
                    key: "{id}",
                    id,
                    position: data.pos,
                    kind: *kind,
                }
            }
        }

        NodeKind::Leaf {
            desc,
            params: _,
            external_receivers,
        } => rsx! {
            Leaf {
                key: "{id}",
                id,
                position: data.pos,
                name: desc.name().clone(),
                kind: desc.kind(),
                receivers: desc.receivers().clone(),
                senders: desc.senders().clone(),
                external_receivers: external_receivers.clone(),
            }
        },
        NodeKind::Root => {
            rsx! {
                Root { key: "{id}", id, position: data.pos }
            }
        }
    });

    rsx! {
        {rendered}
    }
}

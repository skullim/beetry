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
            spec,
            params: _,
            external_receivers,
        } => rsx! {
            Leaf {
                key: "{id}",
                id,
                position: data.pos,
                name: spec.name().clone(),
                kind: spec.kind(),
                receivers: spec.receivers().clone(),
                senders: spec.senders().clone(),
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

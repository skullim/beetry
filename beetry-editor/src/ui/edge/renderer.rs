use dioxus::prelude::*;

use crate::{
    definitions::{Point, PointEdge},
    ui::{
        NodeMap,
        edge::{ContextMenuState, Edge, tracker::Tracker},
    },
};

#[component]
pub(crate) fn Renderer(
    tracker: Signal<Tracker>,
    ui_nodes: ReadSignal<NodeMap>,
    mut edge_context_menu_state: Signal<ContextMenuState>,
) -> Element {
    let nodes_read = ui_nodes.read();
    let tracker_read = tracker.read();

    let edges = tracker_read
        .edges()
        .iter()
        .enumerate()
        .map(|(index, node_edge)| {
            let edge = PointEdge {
                start: nodes_read.get(&node_edge.from).unwrap().pos,
                end: nodes_read.get(&node_edge.to).unwrap().pos,
            };

            rsx! {
                Edge {
                    key: "{index}",
                    edge,
                    edge_index: index,
                    on_context_menu: move |(idx, point): (usize, Point)| {
                        edge_context_menu_state
                            .with_mut(|state| {
                                state.position = point;
                                state.target_edge_index = idx;
                                state.is_visible = true;
                            });
                    },
                }
            }
        });

    rsx! {
        {edges}
    }
}

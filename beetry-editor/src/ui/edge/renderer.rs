use dioxus::prelude::*;

use crate::definitions::{EdgePos, Point};
use crate::editor::ServiceContext;
use crate::signals::RequestRender;
use crate::ui::NodeMap;
use crate::ui::edge::tracker::Tracker;
use crate::ui::edge::{ContextMenuState, Edge};

// Conditions to re-render the edges:
// - new edge created
// - node position updated
#[component]
pub fn Renderer(
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
            let edge_pos = EdgePos {
                start: nodes_read.get(&node_edge.from).unwrap().pos,
                end: nodes_read.get(&node_edge.to).unwrap().pos,
            };

            rsx! {
                Edge {
                    // should be stable across inserts, but not for deletions
                    key: "{index}",
                    pos: edge_pos,
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

#[component]
pub fn Renderer2(
    render_edges: Signal<RequestRender>,
    mut edge_context_menu_state: Signal<ContextMenuState>,
) -> Element {
    debug!("rendering edges");
    let _read = render_edges.read();
    let service = use_context::<ServiceContext>();
    let read = service.service.read();
    let edge_api = read.edge_api();
    let ui_node_api = read.ui_api();
    let edges = edge_api.edges().map(|(id, edge)| {
        //@todo refine on service layer to get position of the port and not node
        //@todo error handling
        let edge_start = ui_node_api.node().data(edge.from).unwrap().position.origin;
        let edge_end = ui_node_api.node().data(edge.to).unwrap().position.origin;
        let edge_pos = EdgePos {
            start: Point {
                x: edge_start.x,
                y: edge_start.y,
            },
            end: Point {
                x: edge_end.x,
                y: edge_end.y,
            },
        };
        rsx! {
            Edge {
                key: "{id}",
                pos: edge_pos,
                edge_index: *id,
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

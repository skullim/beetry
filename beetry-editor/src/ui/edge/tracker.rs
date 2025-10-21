use crate::definitions::{NodeEdge, NodeId};
use indexmap::IndexSet;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub(crate) struct Tracker {
    parent_children_map: HashMap<NodeId, IndexSet<NodeId>>,
    edges: Vec<NodeEdge>,
}

impl Tracker {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn from_edges(edges: Vec<NodeEdge>) -> Self {
        let mut tracker = Self::new();
        for edge in edges {
            tracker.insert(edge);
        }
        tracker
    }

    pub(crate) fn insert(&mut self, edge: NodeEdge) {
        self.edges.push(edge.clone());

        self.parent_children_map
            .entry(edge.from)
            .and_modify(|set| {
                set.insert(edge.to);
            })
            .or_insert_with(|| {
                let mut set = IndexSet::new();
                set.insert(edge.to);
                set
            });
    }

    pub(crate) fn remove_node(&mut self, id: NodeId) {
        self.edges.retain(|edge| edge.from != id && edge.to != id);
        self.parent_children_map.remove(&id);

        for children in self.parent_children_map.values_mut() {
            children.shift_remove(&id);
        }
    }

    pub(crate) fn remove_first_edge_from(&mut self, edge: &NodeEdge) {
        if let Some(idx) = self.edges.iter().position(|e| e.from == edge.from) {
            let removed = self.edges.remove(idx);

            self.parent_children_map
                .entry(removed.from)
                .and_modify(|children| {
                    children.swap_remove(&removed.to);
                });
        }
    }

    pub(crate) fn children_of(&self, id: &NodeId) -> Option<&IndexSet<NodeId>> {
        self.parent_children_map.get(id)
    }

    pub(crate) fn edges(&self) -> &Vec<NodeEdge> {
        &self.edges
    }

    pub(crate) fn has_parent(&self, node_id: NodeId) -> bool {
        self.edges.iter().any(|edge| edge.to == node_id)
    }

    pub(crate) fn has_path(&self, start: NodeId, end: NodeId) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![start];

        while let Some(current) = stack.pop() {
            if current == end {
                return true;
            }

            if visited.insert(current)
                && let Some(children) = self.children_of(&current)
            {
                for &child in children {
                    if !visited.contains(&child) {
                        stack.push(child);
                    }
                }
            }
        }

        false
    }

    pub(crate) fn remove_edge(&mut self, edge_index: usize) -> bool {
        if edge_index >= self.edges.len() {
            return false;
        }
        let edge = &self.edges[edge_index];
        let from = edge.from;
        let to = edge.to;
        self.remove_edge_by_nodes(from, to)
    }

    fn remove_edge_by_nodes(&mut self, from: NodeId, to: NodeId) -> bool {
        if let Some(index) = self
            .edges
            .iter()
            .position(|edge| edge.from == from && edge.to == to)
        {
            let edge = self.edges.remove(index);
            if let Some(children) = self.parent_children_map.get_mut(&edge.from) {
                children.swap_remove(&edge.to);

                // If the parent has no more children, remove it from the map entirely
                if children.is_empty() {
                    self.parent_children_map.remove(&edge.from);
                }
            }

            true
        } else {
            false
        }
    }
}

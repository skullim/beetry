use std::collections::{HashMap, HashSet};

use crate::domain::{
    models::{EdgeId, NodeEdge, NodeId, NodeKind},
    ports::{
        ChannelDataRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        EditorRepositoryViewMut, NodeRepositoryFacadeConcept,
    },
    service::node::{self, NodeService},
};
use anyhow::{Result, anyhow, bail};
use tracing::warn;

/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct EdgeServiceView<'r, 's, NRF, ER, CR> {
    repo: &'r mut EditorRepository<NRF, ER, CR>,
    edge_service: &'s mut EdgeService,
}

impl<'r, 's, NRF, ER, CR> EdgeServiceView<'r, 's, NRF, ER, CR>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelDataRepositoryConcept,
{
    pub(crate) fn new(
        repo: &'r mut EditorRepository<NRF, ER, CR>,
        edge_service: &'s mut EdgeService,
    ) -> Self {
        Self { repo, edge_service }
    }

    pub fn create(&mut self, node_edge: NodeEdge) -> Result<()> {
        let EditorRepositoryViewMut { node, edge, .. } = self.repo.view_mut();
        self.edge_service.create(edge, node, node_edge)
    }

    pub fn parent_of(&self, id: NodeId) -> Option<NodeId> {
        self.edge_service.parent_of(id)
    }

    pub fn children_of(&self, id: NodeId) -> impl Iterator<Item = NodeId> {
        self.edge_service.children_of(id)
    }

    pub fn edges(&self) -> impl Iterator<Item = (EdgeId, &NodeEdge)> {
        EdgeService::edges(self.repo.edge())
    }

    pub fn remove(&mut self, id: EdgeId) -> Result<()> {
        self.edge_service.remove(self.repo.edge_mut(), id)
    }
}

#[derive(Default)]
pub(crate) struct EdgeService {
    // not strictly necessary, but good for performance to cache the tree hierarchy
    parent_children_map: HashMap<NodeId, HashSet<NodeId>>,
    child_parent_map: HashMap<NodeId, NodeId>,
    id_assigner: EdgeIdAssigner,
}

impl EdgeService {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn create(
        &mut self,
        edge_repo: &mut impl EdgeRepositoryConcept,
        node_repo: &impl NodeRepositoryFacadeConcept,
        edge: NodeEdge,
    ) -> Result<()> {
        // 1. Check node types
        // 2. Based on type implement valid connection business logic as all nodes have different rules
        // - each node can only have 1 parent except root that has no parents
        // - root has only one child
        // - control node can have >= 1 children
        // - decorator node has always 1 child
        // - action/condition nodes have no children

        let node_view = node_repo.view();
        let (parent, child) = (edge.from, edge.to);

        NodeService::ensure_exists(node_view, parent)?;
        NodeService::ensure_exists(node_view, child)?;

        // validate parent
        let parent_kind = node::SpecService::kind(node_view.specs, node_view.nodes, parent)?;
        if matches!(parent_kind, NodeKind::Action | NodeKind::Condition) {
            bail!("attempted to create invalid edge: leaf nodes must have no children");
        }

        let child_kind = node::SpecService::kind(node_view.specs, node_view.nodes, child)?;
        if let NodeKind::Root = child_kind {
            bail!("attempted to create invalid edge: root node must not have any parent");
        }

        if self.would_create_cycle(&edge) {
            bail!("attempted to create invalid edge: edge would create a cycle");
        }

        //implicit re-parenting (more convenient to use for client)
        if let Some(old_parent_id) = self.child_parent_map.get(&child).copied()
            && let Some(edge_id) =
                Self::find_edge_id_from(edge_repo, |(_, e)| e.from == old_parent_id)
        {
            warn!("re-parenting node {child} from {old_parent_id} to {parent}");
            self.remove(edge_repo, edge_id)?;
        }

        //implicit re-childing (more convenient to use for client)
        if matches!(parent_kind, NodeKind::Root | NodeKind::Decorator) {
            // root and decorator nodes can only have one child. If there is one, remove it
            if let Some(Some(child_id)) = self
                .parent_children_map
                .get(&parent)
                .map(|children| children.iter().next().copied())
                && let Some(edge_id) = Self::find_edge_id_from(edge_repo, |(_, e)| e.to == child_id)
            {
                self.remove(edge_repo, edge_id)?;
            }
        }

        let id = self.id_assigner.next_id();
        edge_repo.create(id, edge)?;
        self.child_parent_map.insert(child, parent);
        self.parent_children_map
            .entry(parent)
            .or_default()
            .insert(child);
        Ok(())
    }

    // All edges are *always* removed by Id
    fn remove(&mut self, edge_repo: &mut impl EdgeRepositoryConcept, id: EdgeId) -> Result<()> {
        let removed = edge_repo
            .remove(id)
            .ok_or_else(|| anyhow!("attempted to remove edge {id} that does not exist"))?;

        if let Some(children) = self.parent_children_map.get_mut(&removed.from) {
            children.remove(&removed.to);

            // If the parent has no more children, remove it from the map entirely
            // @todo: check if this can be avoided
            if children.is_empty() {
                self.parent_children_map.remove(&removed.from);
            }
        }
        self.child_parent_map.remove(&removed.to);
        Ok(())
    }

    fn parent_of(&self, child_id: NodeId) -> Option<NodeId> {
        self.child_parent_map.get(&child_id).copied()
    }

    fn children_of(&self, parent_id: NodeId) -> impl Iterator<Item = NodeId> {
        self.parent_children_map
            .get(&parent_id)
            .into_iter()
            .flat_map(|children| children.iter().copied())
    }

    fn edges(edge_repo: &impl EdgeRepositoryConcept) -> impl Iterator<Item = (EdgeId, &NodeEdge)> {
        edge_repo.iter()
    }

    pub(crate) fn on_node_removal(
        &mut self,
        edge_repo: &mut impl EdgeRepositoryConcept,
        id: NodeId,
    ) -> Result<()> {
        let filtered: Vec<_> = edge_repo
            .iter()
            .filter_map(|(edge_id, e)| {
                if e.to == id || e.from == id {
                    Some(edge_id)
                } else {
                    None
                }
            })
            .collect();
        for id in filtered {
            self.remove(edge_repo, id)?;
        }
        Ok(())
    }

    fn would_create_cycle(&self, edge: &NodeEdge) -> bool {
        let origin = edge.from;

        let mut nodes_to_visit = vec![edge.to];
        while let Some(node) = nodes_to_visit.pop() {
            if node == origin {
                return true;
            }
            if let Some(children) = self.parent_children_map.get(&node) {
                nodes_to_visit.extend(children);
            }
        }
        false
    }

    fn find_edge_id_from(
        edge_repo: &impl EdgeRepositoryConcept,
        predicate: impl FnMut(&(EdgeId, &NodeEdge)) -> bool,
    ) -> Option<EdgeId> {
        edge_repo.iter().find(predicate).map(|(id, _)| id)
    }
}

#[derive(Default)]
struct EdgeIdAssigner {
    id: EdgeId,
}

impl EdgeIdAssigner {
    fn next_id(&mut self) -> EdgeId {
        let id = self.id;
        self.id += 1;
        id
    }
}

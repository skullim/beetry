use std::collections::{HashMap, HashSet};

use crate::domain::service::node::{SpecServiceApi, TrackerServiceApi};
use crate::domain::{
    models::{EdgeId, NodeEdge, NodeId, NodeKind},
    repository::{EdgeRepositoryConcept, NodeRepositoryFacadeConcept},
};
use anyhow::{Result, anyhow, bail};
use tracing::warn;

/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct EdgeServiceApi<'a, ER, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    edge_repo: &'a mut ER,
    edge_service: &'a mut EdgeService,
    node_tracker_api: TrackerServiceApi<'a, NRF::NodeRepo>,
    node_spec_api: SpecServiceApi<'a, NRF::SpecRepo, NRF::NodeRepo>,
}

impl<'a, ER, NRF> EdgeServiceApi<'a, ER, NRF>
where
    ER: EdgeRepositoryConcept,
    NRF: NodeRepositoryFacadeConcept,
{
    pub(super) fn new(
        edge_repo: &'a mut ER,
        edge_service: &'a mut EdgeService,
        node_tracker_api: TrackerServiceApi<'a, NRF::NodeRepo>,
        node_spec_api: SpecServiceApi<'a, NRF::SpecRepo, NRF::NodeRepo>,
    ) -> Self {
        Self {
            edge_repo,
            edge_service,
            node_tracker_api,
            node_spec_api,
        }
    }

    pub fn create(&mut self, edge: NodeEdge) -> Result<()> {
        self.edge_service.create::<NRF>(
            self.edge_repo,
            &self.node_tracker_api,
            &self.node_spec_api,
            edge,
        )
    }

    pub fn parent_of(&self, id: NodeId) -> Option<&NodeId> {
        self.edge_service.parent_of(id)
    }

    pub fn children_of(&self, id: NodeId) -> impl Iterator<Item = &NodeId> {
        self.edge_service.children_of(id)
    }

    pub fn edges(&self) -> impl Iterator<Item = (&EdgeId, &NodeEdge)> {
        EdgeService::edges(self.edge_repo)
    }

    pub fn remove(&mut self, id: &EdgeId) -> Result<()> {
        self.edge_service.remove(self.edge_repo, id)
    }
}

pub(super) struct OnNodeRemovalServiceApi<'a, ER> {
    service: &'a mut EdgeService,
    repo: &'a mut ER,
}

impl<'a, ER> OnNodeRemovalServiceApi<'a, ER>
where
    ER: EdgeRepositoryConcept,
{
    pub(super) fn new(service: &'a mut EdgeService, repo: &'a mut ER) -> Self {
        Self { service, repo }
    }

    pub(super) fn on_removal(&mut self, id: NodeId) -> Result<()> {
        let filtered: Vec<_> = self
            .repo
            .iter()
            .filter_map(|(edge_id, e)| {
                if e.to == id || e.from == id {
                    Some(*edge_id)
                } else {
                    None
                }
            })
            .collect();
        for id in filtered {
            self.service.remove(self.repo, &id)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub(super) struct EdgeService {
    // not strictly necessary, but good for performance to cache the tree hierarchy
    // @todo replace value type to https://docs.rs/mitsein/latest/mitsein/btree_set1/type.BTreeSet1.html.
    // Rationale: entry should exist only if there is any present child otherwise the entry should be removed
    // Here the exact order is not kept, as it can change dynamically based on the position of any child
    parent_children_map: HashMap<NodeId, HashSet<NodeId>>,
    child_parent_map: HashMap<NodeId, NodeId>,
}

impl EdgeService {
    pub(super) fn new() -> Self {
        Self::default()
    }

    fn create<NRF>(
        &mut self,
        edge_repo: &mut impl EdgeRepositoryConcept,
        node_tracker_api: &TrackerServiceApi<'_, NRF::NodeRepo>,
        node_spec_api: &SpecServiceApi<'_, NRF::SpecRepo, NRF::NodeRepo>,
        edge: NodeEdge,
    ) -> Result<()>
    where
        NRF: NodeRepositoryFacadeConcept,
    {
        // 1. Check node types
        // 2. Based on type implement valid connection business logic as all nodes have different rules
        // - each node can only have 1 parent except root that has no parents
        // - root has only one child
        // - control node can have >= 1 children
        // - decorator node has always 1 child
        // - action/condition nodes have no children

        let (parent, child) = (edge.from, edge.to);
        node_tracker_api.ensure_exists(parent)?;
        node_tracker_api.ensure_exists(child)?;

        // validate parent
        let parent_kind = node_spec_api.kind(parent)?;
        if matches!(parent_kind, NodeKind::Action | NodeKind::Condition) {
            bail!("attempted to create invalid edge: leaf nodes must have no children");
        }

        let child_kind = node_spec_api.kind(child)?;
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
            self.remove(edge_repo, &edge_id)?;
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
                self.remove(edge_repo, &edge_id)?;
            }
        }

        edge_repo.create(edge)?;
        self.child_parent_map.insert(child, parent);
        self.parent_children_map
            .entry(parent)
            .or_default()
            .insert(child);
        Ok(())
    }

    // All edges are *always* removed by Id
    fn remove(&mut self, edge_repo: &mut impl EdgeRepositoryConcept, id: &EdgeId) -> Result<()> {
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

    fn parent_of(&self, child_id: NodeId) -> Option<&NodeId> {
        self.child_parent_map.get(&child_id)
    }

    fn children_of(&self, parent_id: NodeId) -> impl Iterator<Item = &NodeId> {
        self.parent_children_map
            .get(&parent_id)
            .into_iter()
            .flat_map(|children| children.iter())
    }

    fn edges(edge_repo: &impl EdgeRepositoryConcept) -> impl Iterator<Item = (&EdgeId, &NodeEdge)> {
        edge_repo.iter()
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
        predicate: impl FnMut(&(&EdgeId, &NodeEdge)) -> bool,
    ) -> Option<EdgeId> {
        edge_repo.iter().find(predicate).map(|(id, _)| *id)
    }
}

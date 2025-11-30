use std::collections::{HashMap, HashSet};

use crate::domain::{
    models::NodeId,
    ports::{EdgeRepositoryConcept, NodeRepositoryFacadeConcept},
    service::node::NodeService,
};
use anyhow::Result;

struct EdgeService {
    // not strictly necessary, but good for performance to cache the tree hierarchy
    parent_children_map: HashMap<NodeId, HashSet<NodeId>>,
}

impl EdgeService {
    fn connect(
        &mut self,
        edge_repo: &mut impl EdgeRepositoryConcept,
        node_repo: &impl NodeRepositoryFacadeConcept,
        from: NodeId,
        to: NodeId,
    ) -> Result<()> {
        let node_view = node_repo.view();
        NodeService::ensure_exists(node_view, from)?;
        NodeService::ensure_exists(node_view, to)?;
        todo!()
    }

    fn on_node_removal(&mut self, id: NodeId) -> Result<()> {
        todo!()
    }
}

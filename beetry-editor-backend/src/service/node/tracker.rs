use crate::repository::NodeRepository;
use anyhow::{Result, anyhow, bail};
use beetry_editor_types::{
    id::{NodeId, NodeSpecId},
    spec::node::NodeKind,
};

use super::NodeService;

pub struct TrackerView<'a> {
    pub(crate) service: &'a NodeService,
    pub(crate) repo: &'a NodeRepository,
}

impl<'a> TrackerView<'a> {
    pub(crate) fn new(service: &'a NodeService, repo: &'a NodeRepository) -> Self {
        Self { service, repo }
    }

    pub(crate) fn ensure_exists(&self, id: NodeId) -> Result<()> {
        if !self.repo.contains(id) {
            bail!("node {id} does not exist");
        }
        Ok(())
    }
}

pub trait NodeTrackerQuery {
    fn nodes(&self) -> impl Iterator<Item = &NodeId>;
    fn leaf_nodes(&self) -> impl Iterator<Item = &NodeId>;
    fn spec_id(&self, id: NodeId) -> Result<NodeSpecId>;
    fn root_id(&self) -> Result<NodeId>;
    fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId>;
}

impl NodeTrackerQuery for TrackerView<'_> {
    fn nodes(&self) -> impl Iterator<Item = &NodeId> {
        self.repo.ids()
    }

    fn leaf_nodes(&self) -> impl Iterator<Item = &NodeId> {
        self.nodes_by_kind(NodeKind::action())
            .chain(self.nodes_by_kind(NodeKind::condition()))
    }

    fn spec_id(&self, id: NodeId) -> Result<NodeSpecId> {
        self.repo
            .spec_id(id)
            .copied()
            .ok_or_else(|| anyhow!("no mapping between node id {id} and spec id exists"))
    }

    fn root_id(&self) -> Result<NodeId> {
        self.nodes_by_kind(NodeKind::Root)
            .next()
            .copied()
            .ok_or_else(|| anyhow!("no root found in the tree"))
    }

    fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId> {
        self.service.nodes_by_kind(kind)
    }
}

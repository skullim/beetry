use crate::repository::{NodeRepositoryConcept, SpecRepositoryConcept};
use anyhow::{Result, bail};
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};

use beetry_editor_types::{
    id::{NodeId, NodeSpecId},
    spec::node::{NodeKind, NodeSpec, NodeSpecKey},
};

use super::SpecView;

#[derive(Debug, Default)]
pub(crate) struct NodeService {
    spec_cache: HashMap<NodeSpecKey, NodeSpecId>,
    node_cache: HashMap<NodeKind, HashSet<NodeId>>,
}

impl NodeService {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn create(
        &mut self,
        spec_repo: &mut impl SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
        node_repo: &mut impl NodeRepositoryConcept,
        spec: &NodeSpec,
    ) -> Result<NodeId> {
        let kind = spec.kind();
        self.validate_creation(kind)?;
        let spec_id = if let Some(id) = self.spec_cache.get(spec.key()) {
            *id
        } else {
            debug!("inserting new spec into spec repo");
            let spec_id = spec_repo.create(spec.clone())?;
            self.spec_cache.insert(spec.key.clone(), spec_id);
            spec_id
        };

        let id = node_repo.create(spec_id)?;
        self.node_cache.entry(kind).or_default().insert(id);
        Ok(id)
    }

    fn validate_creation(&self, kind: NodeKind) -> Result<()> {
        if let NodeKind::Root = kind
            && let Some(root) = self.node_cache.get(&kind)
            && !root.is_empty()
        {
            bail!("attempted to create multiple roots");
        }
        Ok(())
    }

    pub(crate) fn load_node(
        &mut self,
        spec_repo: &impl SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
        node_repo: &mut impl NodeRepositoryConcept,
        id: NodeId,
        spec_id: NodeSpecId,
    ) -> Result<()> {
        let spec_view = SpecView {
            spec_repo,
            node_repo,
        };
        let kind = spec_view.kind_by_spec_id(spec_id)?;
        self.validate_creation(kind)?;
        node_repo.load(id, spec_id)?;
        self.node_cache.entry(kind).or_default().insert(id);
        Ok(())
    }

    pub(crate) fn load_spec(
        &mut self,
        spec_repo: &mut impl SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
        id: NodeSpecId,
        spec: NodeSpec,
    ) -> Result<()> {
        if let Some(id) = self.spec_cache.get(spec.key()) {
            warn!("spec {id} is already loaded");
        } else {
            spec_repo.load(id, spec.clone())?;
            self.spec_cache.insert(spec.key, id);
        }
        Ok(())
    }

    pub(crate) fn remove(&mut self, spec: &NodeSpec, id: NodeId) -> Result<()> {
        self.node_cache
            .get_mut(&spec.kind())
            .map(|nodes| nodes.remove(&id));
        Ok(())
    }

    pub(crate) fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId> {
        self.node_cache
            .get(&kind)
            .into_iter()
            .flat_map(|nodes| nodes.iter())
    }
}

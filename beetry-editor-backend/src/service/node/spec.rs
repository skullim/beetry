use anyhow::{Context, Result, anyhow};
use beetry_editor_types::{
    id::{NodeId, NodeSpecId},
    spec::node::{NodeKind, NodeName, NodeSpec, ParamsSpec, PortsSpec},
};

use crate::repository::{NodeRepository, NodeSpecRepository};

pub struct SpecView<'a> {
    pub(crate) spec_repo: &'a NodeSpecRepository,
    pub(crate) node_repo: &'a NodeRepository,
}

impl<'a> SpecView<'a> {
    pub(crate) fn new(spec_repo: &'a NodeSpecRepository, node_repo: &'a NodeRepository) -> Self {
        Self {
            spec_repo,
            node_repo,
        }
    }

    pub fn kind(&self, id: NodeId) -> Result<NodeKind> {
        Ok(Self::spec_by_node_id(self.spec_repo, self.node_repo, id)?.kind())
    }

    pub fn params(&self, id: NodeId) -> Result<&ParamsSpec> {
        Self::spec_by_node_id(self.spec_repo, self.node_repo, id)?
            .params()
            .as_ref()
            .ok_or_else(|| anyhow!("expected parameters specification for node {id}"))
    }

    pub fn name_by_spec_id(&self, spec_id: NodeSpecId) -> Result<&NodeName> {
        Ok(Self::spec_by_spec_id(self.spec_repo, spec_id)?.name())
    }

    pub fn kind_by_spec_id(&self, spec_id: NodeSpecId) -> Result<NodeKind> {
        Ok(Self::spec_by_spec_id(self.spec_repo, spec_id)?.kind())
    }

    pub(crate) fn spec_by_node_id<'s>(
        spec_repo: &'s NodeSpecRepository,
        node_repo: &NodeRepository,
        id: NodeId,
    ) -> Result<&'s NodeSpec> {
        let spec_id = *Self::spec_id(node_repo, id)?;
        Self::spec_by_spec_id(spec_repo, spec_id)
            .with_context(|| format!("spec for node id {id} not found"))
    }

    pub fn spec_by_spec_id(
        spec_repo: &NodeSpecRepository,
        spec_id: NodeSpecId,
    ) -> Result<&NodeSpec> {
        spec_repo
            .spec(spec_id)
            .ok_or_else(|| anyhow!("failed to obtain spec {spec_id}"))
    }

    fn spec_id(node_repo: &NodeRepository, id: NodeId) -> Result<&NodeSpecId> {
        node_repo
            .spec_id(id)
            .ok_or_else(|| anyhow!("no mapping between node id {id} and spec id exists"))
    }
}

pub trait SpecBySpecIdQuery {
    fn spec(&self, id: NodeSpecId) -> Result<&NodeSpec>;
    fn name(&self, id: NodeSpecId) -> Result<&NodeName>;
    fn kind(&self, id: NodeSpecId) -> Result<NodeKind>;
}

pub struct SpecBySpecIdQueryView<'a> {
    repo: &'a NodeSpecRepository,
}

impl<'a> SpecBySpecIdQueryView<'a> {
    pub(crate) fn new(repo: &'a NodeSpecRepository) -> Self {
        Self { repo }
    }
}

impl SpecBySpecIdQuery for SpecBySpecIdQueryView<'_> {
    fn spec(&self, id: NodeSpecId) -> Result<&NodeSpec> {
        self.repo
            .spec(id)
            .ok_or_else(|| anyhow!("failed to obtain spec {id}"))
    }
    fn name(&self, id: NodeSpecId) -> Result<&NodeName> {
        Ok(self.spec(id)?.name())
    }
    fn kind(&self, id: NodeSpecId) -> Result<NodeKind> {
        Ok(self.spec(id)?.kind())
    }
}

pub struct SpecByNodeIdQueryView<'a> {
    spec_query: SpecBySpecIdQueryView<'a>,
    node_repo: &'a NodeRepository,
}

impl<'a> SpecByNodeIdQueryView<'a> {
    #[must_use]
    pub fn new(spec_query: SpecBySpecIdQueryView<'a>, node_repo: &'a NodeRepository) -> Self {
        Self {
            spec_query,
            node_repo,
        }
    }

    fn spec_id(&self, id: NodeId) -> Result<&NodeSpecId> {
        self.node_repo
            .spec_id(id)
            .ok_or_else(|| anyhow!("no mapping between node id {id} and spec id exists"))
    }
}

pub trait SpecByNodeIdQuery {
    fn spec(&self, id: NodeId) -> Result<&NodeSpec>;
    fn name(&self, id: NodeId) -> Result<&NodeName>;
    fn kind(&self, id: NodeId) -> Result<NodeKind>;
    fn ports(&self, id: NodeId) -> Result<&PortsSpec>;
    fn params(&self, id: NodeId) -> Result<&ParamsSpec>;
}

impl SpecByNodeIdQuery for SpecByNodeIdQueryView<'_> {
    fn spec(&self, id: NodeId) -> Result<&NodeSpec> {
        let spec_id = self.spec_id(id)?;
        self.spec_query.spec(*spec_id)
    }
    fn name(&self, id: NodeId) -> Result<&NodeName> {
        Ok(self.spec(id)?.name())
    }
    fn kind(&self, id: NodeId) -> Result<NodeKind> {
        Ok(self.spec(id)?.kind())
    }
    fn ports(&self, id: NodeId) -> Result<&PortsSpec> {
        self.spec(id)?
            .ports()
            .as_ref()
            .ok_or_else(|| anyhow!("expected port specification for node {id}"))
    }
    fn params(&self, id: NodeId) -> Result<&ParamsSpec> {
        self.spec(id)?
            .params()
            .as_ref()
            .ok_or_else(|| anyhow!("expected parameters specification for node {id}"))
    }
}

use crate::repository::{NodeRepositoryConcept, SpecRepositoryConcept};
use anyhow::{Context, Result, anyhow};
use beetry_editor_types::{
    id::{NodeId, NodeSpecId},
    spec::node::{NodeKind, NodeName, NodeSpec, ParamsSpec, PortsSpec},
};

pub struct SpecView<'a, SR, NR> {
    pub(crate) spec_repo: &'a SR,
    pub(crate) node_repo: &'a NR,
}

impl<'a, SR, NR> SpecView<'a, SR, NR>
where
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
    NR: NodeRepositoryConcept,
{
    pub(crate) fn new(spec_repo: &'a SR, node_repo: &'a NR) -> Self {
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
        spec_repo: &'s SR,
        node_repo: &NR,
        id: NodeId,
    ) -> Result<&'s NodeSpec> {
        let spec_id = *Self::spec_id(node_repo, id)?;
        Self::spec_by_spec_id(spec_repo, spec_id)
            .with_context(|| format!("spec for node id {id} not found"))
    }

    pub fn spec_by_spec_id(spec_repo: &SR, spec_id: NodeSpecId) -> Result<&NodeSpec> {
        spec_repo
            .spec(spec_id)
            .ok_or_else(|| anyhow!("failed to obtain spec {spec_id}"))
    }

    fn spec_id(node_repo: &NR, id: NodeId) -> Result<&NodeSpecId> {
        node_repo
            .spec_id(&id)
            .ok_or_else(|| anyhow!("no mapping between node id {id} and spec id exists"))
    }
}

pub trait SpecBySpecIdQuery {
    fn spec(&self, id: NodeSpecId) -> Result<&NodeSpec>;
    fn name(&self, id: NodeSpecId) -> Result<&NodeName>;
    fn kind(&self, id: NodeSpecId) -> Result<NodeKind>;
}

pub struct SpecBySpecIdQueryView<'a, SR> {
    repo: &'a SR,
}

impl<'a, SR> SpecBySpecIdQueryView<'a, SR> {
    pub(crate) fn new(repo: &'a SR) -> Self {
        Self { repo }
    }
}

impl<SR> SpecBySpecIdQuery for SpecBySpecIdQueryView<'_, SR>
where
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
{
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

pub struct SpecByNodeIdQueryView<'a, SR, NR> {
    spec_query: SpecBySpecIdQueryView<'a, SR>,
    node_repo: &'a NR,
}

impl<'a, SR, NR> SpecByNodeIdQueryView<'a, SR, NR>
where
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
    NR: NodeRepositoryConcept,
{
    pub fn new(spec_query: SpecBySpecIdQueryView<'a, SR>, node_repo: &'a NR) -> Self {
        Self {
            spec_query,
            node_repo,
        }
    }

    fn spec_id(&self, id: NodeId) -> Result<&NodeSpecId> {
        self.node_repo
            .spec_id(&id)
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

impl<SR, NR> SpecByNodeIdQuery for SpecByNodeIdQueryView<'_, SR, NR>
where
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
    NR: NodeRepositoryConcept,
{
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

use std::collections::{HashMap, HashSet};

use crate::domain::{
    models::{
        NodeChannelPortId, NodeId, NodeKind, NodePortConnection, NodePortKind, NodePortSpec,
        NodePosition, NodeSpec, NodeSpecId,
    },
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        NodePortRepositoryConcept, NodePositionRepositoryConcept, NodeRepositoryConcept,
        NodeRepositoryFacadeConcept, NodeRepositoryFacadeView, NodeRepositoryFacadeViewMut,
        NodeSpecRepositoryConcept, ParamValuesRepositoryConcept,
    },
    service::edge::EdgeService,
};
use anyhow::{Result, anyhow, bail};
use beetry_plugin::ActionSpec;
use beetry_serde::{de::parameter::Parameters, ser::node::NodeName};

/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct NodeServiceView<'r, 's, 'e, NRF, ER, CR> {
    repo: &'r mut EditorRepository<NRF, ER, CR>,
    node_service: &'s mut NodeService,
    edge_service: &'e mut EdgeService,
}

impl<'r, 's, 'e, NRF, ER, CR> NodeServiceView<'r, 's, 'e, NRF, ER, CR>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelRepositoryConcept,
{
    pub(crate) fn new(
        repo: &'r mut EditorRepository<NRF, ER, CR>,
        node_service: &'s mut NodeService,
        edge_service: &'e mut EdgeService,
    ) -> Self {
        Self {
            repo,
            node_service,
            edge_service,
        }
    }

    pub fn create_node(&mut self, spec: NodeSpec) -> Result<NodeId> {
        let view = self.repo.node_mut().view_mut();
        self.node_service.create_node(spec, view)
    }

    pub fn remove(&mut self, id: NodeId) -> Result<()> {
        let repo = &mut self.repo;
        self.node_service.on_node_removal(repo.node_mut(), id)?;
        self.edge_service.on_node_removal(repo.edge_mut(), id)
    }

    pub fn kind(&self, id: NodeId) -> Result<NodeKind> {
        let view = self.repo.node().view();
        NodeService::kind(view.specs, view.nodes, id)
    }

    pub fn name(&self, id: NodeId) -> Result<&NodeName> {
        let view = self.repo.node().view();
        NodeService::name(view.specs, view.nodes, id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = &NodeId> {
        NodeService::nodes(self.repo.node())
    }

    pub fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId> {
        self.node_service.nodes_by_kind(kind)
    }

    pub fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        NodeService::update_position(self.repo.node_mut(), id, position)
    }

    pub fn positions(&self, kind: NodeKind) -> impl Iterator<Item = &NodePosition> {
        self.node_service.positions(self.repo.node(), kind)
    }

    pub fn parameters(&self, id: NodeId) -> Result<&Parameters> {
        NodeService::parameters(self.repo.node(), id)
    }

    pub fn port_ids(&self, node_id: NodeId) -> impl Iterator<Item = NodeChannelPortId> {
        let NodeRepositoryFacadeView { ports, .. } = self.repo.node().view();
        NodeService::port_ids(ports, node_id)
    }

    pub fn port_spec(&self, node_id: NodeId, port_id: NodeChannelPortId) -> Result<&NodePortSpec> {
        let NodeRepositoryFacadeView { ports, .. } = self.repo.node().view();
        NodeService::port_spec(ports, node_id, port_id)
    }
    pub fn port_connection(
        &self,
        node_id: NodeId,
        port_id: NodeChannelPortId,
    ) -> Result<&NodePortConnection> {
        let NodeRepositoryFacadeView { ports, .. } = self.repo.node().view();
        NodeService::port_connection(ports, node_id, port_id)
    }
}

#[derive(Debug, Default)]
pub struct NodeService {
    spec_cache: HashMap<NodeSpec, NodeSpecId>,
    node_cache: HashMap<NodeKind, HashSet<NodeId>>,
}

impl NodeService {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn create_node(
        &mut self,
        spec: NodeSpec,
        view: NodeRepositoryFacadeViewMut<'_, impl NodeRepositoryFacadeConcept>,
    ) -> Result<NodeId> {
        let kind = spec.kind;
        if let NodeKind::Root = spec.kind
            && let Some(root) = self.node_cache.get(&kind)
            && !root.is_empty()
        {
            bail!("attempted to create multiple roots");
        }

        let spec_id = match self.spec_cache.get(&spec) {
            Some(id) => *id,
            None => {
                let spec_id = view.specs.create(spec.clone())?;
                self.spec_cache.insert(spec, spec_id);
                spec_id
            }
        };

        let id = view.nodes.create(spec_id)?;
        self.node_cache.entry(kind).or_default().insert(id);
        Ok(id)
    }

    fn positions<'a>(
        &'a self,
        repo: &'a impl NodeRepositoryFacadeConcept,
        kind: NodeKind,
    ) -> impl Iterator<Item = &'a NodePosition> {
        let view = repo.view();
        let position_ids = self
            .node_cache
            .get(&kind)
            .into_iter()
            .flat_map(|i| i.iter().copied());
        position_ids.flat_map(|id| view.positions.position(id))
    }

    pub(crate) fn kind(
        spec_repo: &impl NodeSpecRepositoryConcept,
        node_repo: &impl NodeRepositoryConcept,
        id: NodeId,
    ) -> Result<NodeKind> {
        let spec_id = Self::spec_id(node_repo, id)?;
        Ok(Self::spec(spec_repo, *spec_id)?.kind)
    }

    pub(crate) fn name<'a>(
        spec_repo: &'a impl NodeSpecRepositoryConcept,
        node_repo: &'a impl NodeRepositoryConcept,
        id: NodeId,
    ) -> Result<&'a NodeName> {
        let spec_id = Self::spec_id(node_repo, id)?;
        Ok(&Self::spec(spec_repo, *spec_id)?.name)
    }

    fn spec_id(node_repo: &impl NodeRepositoryConcept, id: NodeId) -> Result<&NodeSpecId> {
        node_repo
            .spec_id(&id)
            .ok_or_else(|| anyhow!("no mapping between node id {id} and spec id exists"))
    }

    fn spec(repo: &impl NodeSpecRepositoryConcept, id: NodeSpecId) -> Result<&NodeSpec> {
        repo.spec(id)
            .ok_or_else(|| anyhow!("failed to obtain node {id} spec"))
    }

    fn on_node_removal(
        &mut self,
        repo: &mut impl NodeRepositoryFacadeConcept,
        id: NodeId,
    ) -> Result<()> {
        let view = repo.view_mut();
        let spec = Self::spec(view.specs, id)?;
        self.node_cache
            .get_mut(&spec.kind)
            .map(|nodes| nodes.remove(&id));

        view.positions.remove(id)?;
        view.parameters.remove(id)
    }

    pub(crate) fn ensure_exists(
        view: NodeRepositoryFacadeView<'_, impl NodeRepositoryFacadeConcept>,
        id: NodeId,
    ) -> Result<()> {
        if !view.nodes.contains(&id) {
            bail!("node {id} does not exist");
        }
        Ok(())
    }

    fn initialize_ports(
        ports_repo: &mut impl NodePortRepositoryConcept,
        id: NodeId,
        spec: &ActionSpec,
    ) -> Result<()> {
        let mut port_specs = vec![];
        for msg_spec in &spec.schema.senders {
            port_specs.push(NodePortSpec {
                kind: NodePortKind::Sender,
                msg_spec: msg_spec.clone(),
            });
        }
        for msg_spec in &spec.schema.receivers {
            port_specs.push(NodePortSpec {
                kind: NodePortKind::Receiver,
                msg_spec: msg_spec.clone(),
            });
        }

        Self::create_node_ports(ports_repo, id, port_specs.into_iter())?;
        // collect to avoid borrowing mutably in the for loop
        let port_ids: Vec<_> = Self::port_ids(ports_repo, id).collect();
        for port_id in port_ids {
            Self::connect_port(ports_repo, id, port_id, NodePortConnection::default())?;
        }
        Ok(())
    }

    fn nodes(repo: &impl NodeRepositoryFacadeConcept) -> impl Iterator<Item = &NodeId> {
        let view = repo.view();
        view.nodes.nodes()
    }

    pub fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId> {
        self.node_cache
            .get(&kind)
            .into_iter()
            .flat_map(|nodes| nodes.iter())
    }

    fn update_position(
        repo: &mut impl NodeRepositoryFacadeConcept,
        id: NodeId,
        position: NodePosition,
    ) -> Result<()> {
        Self::ensure_exists(repo.view(), id)?;
        let NodeRepositoryFacadeViewMut { positions, .. } = repo.view_mut();
        positions.update(id, position)
    }

    fn insert_parameter() {
        todo!()
    }

    fn parameters(repo: &impl NodeRepositoryFacadeConcept, id: NodeId) -> Result<&Parameters> {
        let view = repo.view();
        view.parameters
            .params(id)
            .ok_or_else(|| anyhow!("failed to obtain parameters for node {id}"))
    }

    fn create_node_ports(
        repo: &mut impl NodePortRepositoryConcept,
        node_id: NodeId,
        port_specs: impl Iterator<Item = NodePortSpec>,
    ) -> Result<()> {
        repo.create(node_id, port_specs)
    }

    pub(crate) fn connect_port(
        repo: &mut impl NodePortRepositoryConcept,
        node_id: NodeId,
        port_id: NodeChannelPortId,
        kind: NodePortConnection,
    ) -> Result<()> {
        repo.set_conn(node_id, port_id, kind)
    }

    pub(crate) fn port_ids(
        repo: &impl NodePortRepositoryConcept,
        node_id: NodeId,
    ) -> impl Iterator<Item = NodeChannelPortId> {
        repo.ports(node_id)
    }

    pub(crate) fn port_spec(
        repo: &impl NodePortRepositoryConcept,
        node_id: NodeId,
        port_id: NodeChannelPortId,
    ) -> Result<&NodePortSpec> {
        repo.spec(node_id, port_id)
            .ok_or_else(|| anyhow!("unable to retrieve port spec"))
    }

    pub(crate) fn port_connection(
        repo: &impl NodePortRepositoryConcept,
        node_id: NodeId,
        port_id: NodeChannelPortId,
    ) -> Result<&NodePortConnection> {
        repo.connection(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) connection")
        })
    }
}

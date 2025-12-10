use std::collections::{HashMap, HashSet};

use crate::domain::{
    models::{
        NodeId, NodeKind, NodePortConnection, NodePortId, NodePortKind, NodePortSpec, NodePosition,
        NodeSpec, NodeSpecId, PortsSpec,
    },
    ports::{
        ChannelDataRepositoryConcept, ChannelRepositoryFacadeConcept, EdgeRepositoryConcept,
        EditorRepository, NodePositionRepositoryConcept, NodeRepositoryConcept,
        NodeRepositoryFacadeConcept, NodeRepositoryFacadeView, NodeRepositoryFacadeViewMut,
        ParamValuesRepositoryConcept, PortStateRepositoryConcept, SpecRepositoryConcept,
    },
    service::edge::EdgeService,
};
use anyhow::{Result, anyhow, bail};
use beetry_plugin::ActionSpec;
use beetry_serde::{de::parameter::Parameters, ser::node::NodeName};

/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct NodeServiceView<'f, 's, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    facade: NodeRepositoryFacadeViewMut<'f, NRF>,
    service: &'s mut NodeService,
}

impl<'f, 's, NRF> NodeServiceView<'f, 's, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    pub(crate) fn new(
        facade: NodeRepositoryFacadeViewMut<'f, NRF>,
        service: &'s mut NodeService,
    ) -> Self {
        Self { facade, service }
    }

    pub fn create_node(&mut self, spec: NodeSpec) -> Result<NodeId> {
        self.service.create(&mut self.facade, spec)
    }

    // pub fn remove(&mut self, id: NodeId) -> Result<()> {
    //     let repo = &mut self.repo;
    //     self.node_service.on_node_removal(repo.node_mut(), id)?;
    //     self.edge_service.on_node_removal(repo.edge_mut(), id)
    // }

    pub fn kind(&self, id: NodeId) -> Result<NodeKind> {
        SpecService::kind(self.facade.specs, self.facade.nodes, id)
    }

    pub fn name(&self, id: NodeId) -> Result<&NodeName> {
        SpecService::name(self.facade.specs, self.facade.nodes, id)
    }

    pub fn ports_spec(&self, id: NodeId) -> Result<&PortsSpec> {
        SpecService::ports(self.facade.specs, self.facade.nodes, id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = &NodeId> {
        NodeService::nodes(self.facade.nodes)
    }

    pub fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId> {
        self.service.nodes_by_kind(kind)
    }

    pub fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        NodeService::update_position(self.facade.nodes, self.facade.positions, id, position)
    }

    pub fn positions(&self, kind: NodeKind) -> impl Iterator<Item = &NodePosition> {
        self.service.positions(self.facade.positions, kind)
    }

    pub fn parameters(&self, id: NodeId) -> Result<&Parameters> {
        NodeService::parameters(self.facade.parameters, id)
    }
}

pub struct SpecService;

impl SpecService {
    pub(crate) fn name<'a>(
        spec_repo: &'a impl SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
        node_repo: &impl NodeRepositoryConcept,
        id: NodeId,
    ) -> Result<&'a NodeName> {
        Ok(&Self::spec(spec_repo, node_repo, id)?.name)
    }

    pub(crate) fn kind(
        spec_repo: &impl SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
        node_repo: &impl NodeRepositoryConcept,
        id: NodeId,
    ) -> Result<NodeKind> {
        Ok(Self::spec(spec_repo, node_repo, id)?.kind)
    }

    pub(crate) fn ports<'a>(
        spec_repo: &'a impl SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
        node_repo: &impl NodeRepositoryConcept,
        id: NodeId,
    ) -> Result<&'a PortsSpec> {
        Ok(&Self::spec(spec_repo, node_repo, id)?.ports)
    }

    fn spec<'a>(
        spec_repo: &'a impl SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
        node_repo: &impl NodeRepositoryConcept,
        id: NodeId,
    ) -> Result<&'a NodeSpec> {
        let spec_id = *Self::spec_id(node_repo, id)?;
        spec_repo
            .spec(spec_id)
            .ok_or_else(|| anyhow!("failed to obtain spec {spec_id} for node {id}"))
    }

    fn spec_id(node_repo: &impl NodeRepositoryConcept, id: NodeId) -> Result<&NodeSpecId> {
        node_repo
            .spec_id(&id)
            .ok_or_else(|| anyhow!("no mapping between node id {id} and spec id exists"))
    }
}

pub struct ParameterValuesService;
pub struct PortStateService;

pub struct PositionService;

pub struct LifecycleService {
    node_cache: HashMap<NodeKind, HashSet<NodeId>>,
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

    pub(crate) fn create(
        &mut self,
        view: &mut NodeRepositoryFacadeViewMut<'_, impl NodeRepositoryFacadeConcept>,
        spec: NodeSpec,
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
        repo: &'a impl NodePositionRepositoryConcept,
        kind: NodeKind,
    ) -> impl Iterator<Item = &'a NodePosition> {
        let position_ids = self
            .node_cache
            .get(&kind)
            .into_iter()
            .flat_map(|i| i.iter().copied());
        position_ids.flat_map(|id| repo.position(id))
    }

    fn on_node_removal(
        &mut self,
        repo: &mut impl NodeRepositoryFacadeConcept,
        id: NodeId,
    ) -> Result<()> {
        let view = repo.view_mut();
        let spec = SpecService::spec(view.specs, view.nodes, id)?;
        self.node_cache
            .get_mut(&spec.kind)
            .map(|nodes| nodes.remove(&id));

        view.positions.remove(id)?;
        view.parameters.remove(id)
    }

    pub(crate) fn ensure_exists(repo: &impl NodeRepositoryConcept, id: NodeId) -> Result<()> {
        if !repo.contains(&id) {
            bail!("node {id} does not exist");
        }
        Ok(())
    }

    // fn initialize_ports(
    //     ports_repo: &mut impl PortStateRepositoryConcept,
    //     id: NodeId,
    //     spec: &ActionSpec,
    // ) -> Result<()> {
    //     let mut port_specs = vec![];
    //     for msg_spec in &spec.schema.senders {
    //         port_specs.push(NodePortSpec {
    //             kind: NodePortKind::Sender,
    //             msg_spec: msg_spec.clone(),
    //         });
    //     }
    //     for msg_spec in &spec.schema.receivers {
    //         port_specs.push(NodePortSpec {
    //             kind: NodePortKind::Receiver,
    //             msg_spec: msg_spec.clone(),
    //         });
    //     }

    //     Self::create_node_ports(ports_repo, id, port_specs.into_iter())?;
    //     // collect to avoid borrowing mutably in the for loop
    //     let port_ids: Vec<_> = Self::port_ids(ports_repo, id).collect();
    //     for port_id in port_ids {
    //         Self::connect_port(ports_repo, id, port_id, NodePortConnection::default())?;
    //     }
    //     Ok(())
    // }

    fn nodes(repo: &impl NodeRepositoryConcept) -> impl Iterator<Item = &NodeId> {
        repo.nodes()
    }

    pub fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId> {
        self.node_cache
            .get(&kind)
            .into_iter()
            .flat_map(|nodes| nodes.iter())
    }

    fn update_position(
        node_repo: &impl NodeRepositoryConcept,
        position_repo: &mut impl NodePositionRepositoryConcept,
        id: NodeId,
        position: NodePosition,
    ) -> Result<()> {
        Self::ensure_exists(node_repo, id)?;
        position_repo.update(id, position)
    }

    fn parameters(repo: &impl ParamValuesRepositoryConcept, id: NodeId) -> Result<&Parameters> {
        repo.params(id)
            .ok_or_else(|| anyhow!("failed to obtain parameters for node {id}"))
    }

    // fn create_node_ports(
    //     repo: &mut impl PortStateRepositoryConcept,
    //     node_id: NodeId,
    //     port_specs: impl Iterator<Item = NodePortSpec>,
    // ) -> Result<()> {
    //     repo.create(node_id, port_specs)
    // }

    // pub(crate) fn connect_port(
    //     repo: &mut impl PortStateRepositoryConcept,
    //     node_id: NodeId,
    //     port_id: NodePortId,
    //     kind: NodePortConnection,
    // ) -> Result<()> {
    //     repo.set_conn(node_id, port_id, kind)
    // }

    // pub(crate) fn port_ids(
    //     repo: &impl PortStateRepositoryConcept,
    //     node_id: NodeId,
    // ) -> impl Iterator<Item = NodePortId> {
    //     repo.ports(node_id)
    // }

    // pub(crate) fn port_connection(
    //     repo: &impl PortStateRepositoryConcept,
    //     node_id: NodeId,
    //     port_id: NodePortId,
    // ) -> Result<&NodePortConnection> {
    //     repo.connection(node_id, port_id).ok_or_else(|| {
    //         anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) connection")
    //     })
    // }
}

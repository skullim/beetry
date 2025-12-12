use std::collections::{HashMap, HashSet};

use crate::domain::{
    models::{
        NodeId, NodeKind, NodePortConnection, NodePortId, NodePosition, NodeSpec, NodeSpecId,
        PortsSpec,
    },
    ports::{
        NodePositionRepositoryConcept, NodeRepositoryConcept, NodeRepositoryFacadeConcept,
        NodeRepositoryFacadeViewMut, ParamValuesRepositoryConcept, PortStateRepositoryConcept,
        SpecRepositoryConcept,
    },
};
use anyhow::{Result, anyhow, bail};
use beetry_serde::{de::parameter::Parameters, ser::node::NodeName};

/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct NodeServiceApi<'f, 's, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    facade: NodeRepositoryFacadeViewMut<'f, NRF>,
    service: &'s mut NodeService,
}

impl<'f, 's, NRF> NodeServiceApi<'f, 's, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    pub(super) fn new(
        facade: NodeRepositoryFacadeViewMut<'f, NRF>,
        service: &'s mut NodeService,
    ) -> Self {
        Self { facade, service }
    }

    pub fn lifecycle_service(&mut self) -> NodeLifecycleApi<'_, NRF::SpecRepo, NRF::NodeRepo> {
        NodeLifecycleApi {
            service: self.service,
            spec_repo: self.facade.specs,
            node_repo: self.facade.nodes,
        }
    }

    //@todo consider merging with lifecycle
    pub fn tracker_service(&self) -> TrackerServiceApi<'_, NRF::NodeRepo> {
        TrackerServiceApi {
            repo: self.facade.nodes,
        }
    }

    // pub fn create_node(&mut self, spec: NodeSpec) -> Result<NodeId> {
    //     self.service.create(&mut self.facade, spec)
    // }

    pub fn port_state_service(&mut self) -> PortStateServiceApi<'_, NRF::PortStateRepo> {
        PortStateServiceApi {
            repo: self.facade.ports,
        }
    }

    // pub fn remove(&mut self, id: NodeId) -> Result<()> {
    //     let repo = &mut self.repo;
    //     self.node_service.on_node_removal(repo.node_mut(), id)?;
    //     self.edge_service.on_node_removal(repo.edge_mut(), id)
    // }

    pub fn spec_service(&self) -> SpecServiceApi<'_, NRF::SpecRepo, NRF::NodeRepo> {
        SpecServiceApi {
            spec_repo: self.facade.specs,
            node_repo: self.facade.nodes,
        }
    }

    // pub fn kind(&self, id: NodeId) -> Result<NodeKind> {
    //     SpecService::kind(self.facade.specs, self.facade.nodes, id)
    // }

    // pub fn name(&self, id: NodeId) -> Result<&NodeName> {
    //     SpecService::name(self.facade.specs, self.facade.nodes, id)
    // }

    // pub fn ports_spec(&self, id: NodeId) -> Result<&PortsSpec> {
    //     SpecService::ports(self.facade.specs, self.facade.nodes, id)
    // }

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

pub struct SpecServiceApi<'a, SR, NR> {
    spec_repo: &'a SR,
    node_repo: &'a NR,
}

impl<'a, SR, NR> SpecServiceApi<'a, SR, NR>
where
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
    NR: NodeRepositoryConcept,
{
    pub(super) fn new(spec_repo: &'a SR, node_repo: &'a NR) -> Self {
        Self {
            spec_repo,
            node_repo,
        }
    }

    pub fn name(&self, id: NodeId) -> Result<&NodeName> {
        Ok(&Self::spec(self.spec_repo, self.node_repo, id)?.name)
    }

    pub fn kind(&self, id: NodeId) -> Result<NodeKind> {
        Ok(Self::spec(self.spec_repo, self.node_repo, id)?.kind)
    }

    pub fn ports(&self, id: NodeId) -> Result<&PortsSpec> {
        Ok(&Self::spec(self.spec_repo, self.node_repo, id)?.ports)
    }

    fn spec<'s>(spec_repo: &'s SR, node_repo: &NR, id: NodeId) -> Result<&'s NodeSpec> {
        let spec_id = *Self::spec_id(node_repo, id)?;
        spec_repo
            .spec(spec_id)
            .ok_or_else(|| anyhow!("failed to obtain spec {spec_id} for node {id}"))
    }

    fn spec_id(node_repo: &NR, id: NodeId) -> Result<&NodeSpecId> {
        node_repo
            .spec_id(&id)
            .ok_or_else(|| anyhow!("no mapping between node id {id} and spec id exists"))
    }
}

pub struct ParameterValuesService;

pub struct PortStateServiceApi<'a, R> {
    repo: &'a mut R,
}

impl<'a, R> PortStateServiceApi<'a, R>
where
    R: PortStateRepositoryConcept,
{
    pub(super) fn new(repo: &'a mut R) -> Self {
        Self { repo }
    }

    pub(crate) fn state(
        &self,
        node_id: NodeId,
        port_id: NodePortId,
    ) -> Result<&NodePortConnection> {
        self.repo.state(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) state")
        })
    }

    pub(crate) fn state_mut(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
    ) -> Result<&mut NodePortConnection> {
        self.repo.state_mut(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) state")
        })
    }
}

pub struct PositionService;

pub struct NodeLifecycleApi<'a, SR, NR> {
    service: &'a mut NodeService,
    spec_repo: &'a mut SR,
    node_repo: &'a mut NR,
}

impl<'a, SR, NR> NodeLifecycleApi<'a, SR, NR>
where
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
    NR: NodeRepositoryConcept,
{
    fn create(&mut self, spec: NodeSpec) -> Result<NodeId> {
        self.service.create(self.spec_repo, self.node_repo, spec)
    }
}

pub struct TrackerServiceApi<'a, NR> {
    repo: &'a NR,
}

impl<'a, NR> TrackerServiceApi<'a, NR>
where
    NR: NodeRepositoryConcept,
{
    pub(super) fn new(repo: &'a NR) -> Self {
        Self { repo }
    }

    pub fn ensure_exists(&self, id: NodeId) -> Result<()> {
        if !self.repo.contains(&id) {
            bail!("node {id} does not exist");
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub(super) struct NodeService {
    spec_cache: HashMap<NodeSpec, NodeSpecId>,
    node_cache: HashMap<NodeKind, HashSet<NodeId>>,
}

impl NodeService {
    pub(super) fn new() -> Self {
        Self::default()
    }

    fn create(
        &mut self,
        spec_repo: &mut impl SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
        node_repo: &mut impl NodeRepositoryConcept,
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
                let spec_id = spec_repo.create(spec.clone())?;
                self.spec_cache.insert(spec, spec_id);
                spec_id
            }
        };

        let id = node_repo.create(spec_id)?;
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
        let spec = SpecServiceApi::spec(view.specs, view.nodes, id)?;
        self.node_cache
            .get_mut(&spec.kind)
            .map(|nodes| nodes.remove(&id));

        view.positions.remove(id)?;
        view.parameters.remove(id)
    }

    fn ensure_exists(repo: &impl NodeRepositoryConcept, id: NodeId) -> Result<()> {
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

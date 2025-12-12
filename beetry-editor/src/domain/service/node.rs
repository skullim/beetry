use std::collections::{HashMap, HashSet};

use crate::domain::{
    models::{
        NodeId, NodeKind, NodePortConnection, NodePortId, NodePosition, NodeSpec, NodeSpecId,
        PortsSpec,
    },
    repository::{
        EdgeRepositoryConcept, NodePositionRepositoryConcept, NodeRepositoryConcept,
        NodeRepositoryFacadeConcept, NodeRepositoryFacadeViewMut, ParamValuesRepositoryConcept,
        PortStateRepositoryConcept, SpecRepositoryConcept,
    },
    service::edge::{self, EdgeService, OnNodeRemovalService},
};
use anyhow::{Result, anyhow, bail};
use beetry_serde::{de::parameter::Parameters, ser::node::NodeName};

/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct NodeServiceApi<'a, NRF, ER>
where
    NRF: NodeRepositoryFacadeConcept,
{
    facade_view: NodeRepositoryFacadeViewMut<'a, NRF>,
    node_service: &'a mut NodeService,
    edge_repo: &'a mut ER,
    edge_service: &'a mut EdgeService,
}

impl<'a, NRF, ER> NodeServiceApi<'a, NRF, ER>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
{
    pub(super) fn new(
        facade_view: NodeRepositoryFacadeViewMut<'a, NRF>,
        node_service: &'a mut NodeService,
        edge_repo: &'a mut ER,
        edge_service: &'a mut EdgeService,
    ) -> Self {
        Self {
            facade_view,
            node_service,
            edge_repo,
            edge_service,
        }
    }

    pub fn spec(&self) -> SpecServiceApi<'_, NRF::SpecRepo, NRF::NodeRepo> {
        SpecServiceApi {
            spec_repo: self.facade_view.specs,
            node_repo: self.facade_view.nodes,
        }
    }

    pub fn lifecycle(&'a mut self) -> NodeLifecycleApi<'a, NRF, ER> {
        NodeLifecycleApi {
            service: self.node_service,
            facade_view: &mut self.facade_view,
            edge_on_node_removal: OnNodeRemovalService::new(self.edge_service, self.edge_repo),
        }
    }

    pub fn tracker(&self) -> TrackerServiceApi<'_, NRF::NodeRepo> {
        TrackerServiceApi {
            service: self.node_service,
            repo: self.facade_view.nodes,
        }
    }

    pub fn port_state(&mut self) -> PortStateServiceApi<'_, NRF::PortStateRepo> {
        PortStateServiceApi {
            repo: self.facade_view.ports,
        }
    }

    pub fn position(&mut self) -> PositionServiceApi<'_, NRF::PositionRepo, NRF::NodeRepo> {
        PositionServiceApi {
            service: self.node_service,
            position_repo: self.facade_view.positions,
            tracker_service: TrackerServiceApi {
                service: self.node_service,
                repo: self.facade_view.nodes,
            },
        }
    }

    pub fn parameters(&mut self) -> ParameterValueServiceApi<'_, NRF::ParamValuesRepo> {
        ParameterValueServiceApi {
            repo: self.facade_view.parameters,
        }
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

pub struct NodeLifecycleApi<'a, NRF, ER>
where
    NRF: NodeRepositoryFacadeConcept,
{
    service: &'a mut NodeService,
    facade_view: &'a mut NodeRepositoryFacadeViewMut<'a, NRF>,
    edge_on_node_removal: edge::OnNodeRemovalService<'a, ER>,
}

impl<'a, NRF, ER> NodeLifecycleApi<'a, NRF, ER>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
{
    pub fn create(&mut self, spec: NodeSpec) -> Result<NodeId> {
        self.service
            .create(self.facade_view.specs, self.facade_view.nodes, spec)
    }

    pub fn remove(&mut self, id: NodeId) -> Result<()> {
        self.service.remove::<NRF>(self.facade_view, id)?;
        self.edge_on_node_removal.on_removal(id)?;
        Ok(())
    }
}

pub struct TrackerServiceApi<'a, NR> {
    service: &'a NodeService,
    repo: &'a NR,
}

impl<'a, NR> TrackerServiceApi<'a, NR>
where
    NR: NodeRepositoryConcept,
{
    pub(super) fn new(service: &'a NodeService, repo: &'a NR) -> Self {
        Self { service, repo }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &NodeId> {
        self.repo.nodes()
    }

    pub fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId> {
        self.service.nodes_by_kind(kind)
    }

    pub(super) fn ensure_exists(&self, id: NodeId) -> Result<()> {
        if !self.repo.contains(&id) {
            bail!("node {id} does not exist");
        }
        Ok(())
    }
}

pub struct ParameterValueServiceApi<'a, PVR> {
    repo: &'a mut PVR,
}

impl<'a, PVR> ParameterValueServiceApi<'a, PVR>
where
    PVR: ParamValuesRepositoryConcept,
{
    pub fn parameters(&self, id: NodeId) -> Result<&Parameters> {
        self.repo
            .params(id)
            .ok_or_else(|| anyhow!("failed to obtain parameters for node {id}"))
    }

    //@todo add API to set parameters, also validate against schema here
}

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

    pub fn state(&self, node_id: NodeId, port_id: NodePortId) -> Result<&NodePortConnection> {
        self.repo.state(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) state")
        })
    }

    pub fn state_mut(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
    ) -> Result<&mut NodePortConnection> {
        self.repo.state_mut(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) state")
        })
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

pub struct PositionServiceApi<'a, PR, NR> {
    service: &'a NodeService,
    tracker_service: TrackerServiceApi<'a, NR>,
    position_repo: &'a mut PR,
}

impl<'a, PR, NR> PositionServiceApi<'a, PR, NR>
where
    PR: NodePositionRepositoryConcept,
    NR: NodeRepositoryConcept,
{
    pub fn positions_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodePosition> {
        self.service.positions_by_kind(self.position_repo, kind)
    }

    pub fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        self.tracker_service.ensure_exists(id)?;
        self.position_repo.update(id, position)
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

    fn positions_by_kind<'a>(
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

    fn remove<NRF>(
        &mut self,
        view: &mut NodeRepositoryFacadeViewMut<'_, NRF>,
        id: NodeId,
    ) -> Result<()>
    where
        NRF: NodeRepositoryFacadeConcept,
    {
        let spec = SpecServiceApi::spec(view.specs, view.nodes, id)?;
        self.node_cache
            .get_mut(&spec.kind)
            .map(|nodes| nodes.remove(&id));

        view.positions.remove(id)?;
        view.parameters.remove(id)
    }

    fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId> {
        self.node_cache
            .get(&kind)
            .into_iter()
            .flat_map(|nodes| nodes.iter())
    }
}

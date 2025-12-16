use crate::domain::{
    models::{
        ChannelId, NodeId, NodeKind, NodePortConnection, NodePortId, NodePosition, NodeSpec,
        NodeSpecId, PortsSpec,
    },
    persistence::{NodePortRecord, NodeRecord, NodeSpecRecord},
    repository::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, NodePositionRepositoryConcept,
        NodeRepositoryConcept, NodeRepositoryFacadeConcept, NodeRepositoryFacadeViewMut,
        ParamValuesRepositoryConcept, PortStateRepositoryConcept, SpecRepositoryConcept,
    },
    service::{
        channel::{ChannelService, ChannelServiceApi, ConnectionContext},
        edge::{self, EdgeService, OnNodeRemovalServiceApi},
    },
};
use anyhow::{Context, Result, anyhow, bail};
use beetry_plugin_types::node::NodeName;
use beetry_reconstruction_types::parameter::Parameters;
use std::collections::{HashMap, HashSet};
use tracing::warn;

/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct NodeServiceApi<'a, NRF, ER, CRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    facade_view: NodeRepositoryFacadeViewMut<'a, NRF>,
    node_service: &'a mut NodeService,
    edge_repo: &'a mut ER,
    edge_service: &'a mut EdgeService,
    channel_facade: &'a mut CRF,
    channel_service: &'a mut ChannelService,
}

impl<'a, NRF, ER, CRF> NodeServiceApi<'a, NRF, ER, CRF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    pub(super) fn new(
        facade_view: NodeRepositoryFacadeViewMut<'a, NRF>,
        node_service: &'a mut NodeService,
        edge_repo: &'a mut ER,
        edge_service: &'a mut EdgeService,
        channel_facade: &'a mut CRF,
        channel_service: &'a mut ChannelService,
    ) -> Self {
        Self {
            facade_view,
            node_service,
            edge_repo,
            edge_service,
            channel_facade,
            channel_service,
        }
    }

    pub fn spec(&self) -> SpecServiceApi<'_, NRF::SpecRepo, NRF::NodeRepo> {
        SpecServiceApi {
            spec_repo: self.facade_view.specs,
            node_repo: self.facade_view.nodes,
        }
    }

    pub fn lifecycle(&'a mut self) -> NodeLifecycleApi<'a, NRF, CRF, ER> {
        NodeLifecycleApi {
            node_service: self.node_service,
            channel_facade: self.channel_facade,
            channel_service: self.channel_service,
            node_facade_view: &mut self.facade_view,
            edge_removal_service_api: OnNodeRemovalServiceApi::new(
                self.edge_service,
                self.edge_repo,
            ),
        }
    }

    pub fn tracker(&self) -> TrackerServiceApi<'_, NRF::NodeRepo> {
        TrackerServiceApi {
            service: self.node_service,
            repo: self.facade_view.nodes,
        }
    }

    pub fn port_state(
        &mut self,
    ) -> PortConnectionServiceApi<'_, NRF::PortStateRepo, NRF::SpecRepo, NRF::NodeRepo, CRF> {
        PortConnectionServiceApi {
            repo: self.facade_view.ports,
            spec_service_api: SpecServiceApi {
                spec_repo: self.facade_view.specs,
                node_repo: self.facade_view.nodes,
            },
            channel_service_api: ChannelServiceApi::new(
                self.channel_facade.view_mut(),
                self.channel_service,
            ),
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
        Ok(&Self::spec_by_node_id(self.spec_repo, self.node_repo, id)?.name)
    }

    pub fn kind(&self, id: NodeId) -> Result<NodeKind> {
        Ok(Self::spec_by_node_id(self.spec_repo, self.node_repo, id)?.kind)
    }

    pub fn ports(&self, id: NodeId) -> Result<&PortsSpec> {
        Ok(&Self::spec_by_node_id(self.spec_repo, self.node_repo, id)?.ports)
    }

    fn kind_by_spec_id(&self, spec_id: NodeSpecId) -> Result<NodeKind> {
        Ok(Self::spec_by_spec_id(self.spec_repo, spec_id)?.kind)
    }

    fn spec_by_node_id<'s>(spec_repo: &'s SR, node_repo: &NR, id: NodeId) -> Result<&'s NodeSpec> {
        let spec_id = *Self::spec_id(node_repo, id)?;
        Self::spec_by_spec_id(spec_repo, spec_id)
            .with_context(|| format!("spec for node id {id} not found"))
    }

    fn spec_by_spec_id(spec_repo: &SR, spec_id: NodeSpecId) -> Result<&NodeSpec> {
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

pub(super) struct LoadNodeApi<'a, NRF, CRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    node_service: &'a mut NodeService,
    channel_service: &'a mut ChannelService,
    node_facade_view: NodeRepositoryFacadeViewMut<'a, NRF>,
    channel_facade: &'a mut CRF,
}

impl<'a, NRF, CRF> LoadNodeApi<'a, NRF, CRF>
where
    NRF: NodeRepositoryFacadeConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    pub(super) fn new(
        node_service: &'a mut NodeService,
        channel_service: &'a mut ChannelService,
        node_facade_view: NodeRepositoryFacadeViewMut<'a, NRF>,
        channel_facade: &'a mut CRF,
    ) -> Self {
        Self {
            node_service,
            channel_service,
            node_facade_view,
            channel_facade,
        }
    }

    pub(super) fn load_node(&mut self, record: NodeRecord) -> Result<()> {
        self.node_service.load_node(
            self.node_facade_view.specs,
            self.node_facade_view.nodes,
            record.id,
            record.spec_id,
        )?;
        let spec_service_api = SpecServiceApi {
            spec_repo: self.node_facade_view.specs,
            node_repo: self.node_facade_view.nodes,
        };
        let channel_service_api =
            ChannelServiceApi::new(self.channel_facade.view_mut(), self.channel_service);
        let mut port_service_api = PortConnectionServiceApi::new(
            self.node_facade_view.ports,
            spec_service_api,
            channel_service_api,
        );
        port_service_api.load(record.id, record.port_records)?;
        if let Some(params) = record.parameters {
            let mut params_service_api = ParameterValueServiceApi {
                repo: self.node_facade_view.parameters,
            };
            params_service_api.load(record.id, params)?;
        }
        Ok(())
    }

    pub(super) fn load_spec(&mut self, record: NodeSpecRecord) -> Result<()> {
        self.node_service
            .load_spec(self.node_facade_view.specs, record)
    }
}

pub struct NodeLifecycleApi<'a, NRF, CRF, ER>
where
    NRF: NodeRepositoryFacadeConcept,
{
    node_service: &'a mut NodeService,
    channel_service: &'a mut ChannelService,
    node_facade_view: &'a mut NodeRepositoryFacadeViewMut<'a, NRF>,
    channel_facade: &'a mut CRF,
    edge_removal_service_api: edge::OnNodeRemovalServiceApi<'a, ER>,
}

impl<'a, NRF, CRF, ER> NodeLifecycleApi<'a, NRF, CRF, ER>
where
    NRF: NodeRepositoryFacadeConcept,
    CRF: ChannelRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
{
    pub fn create(&mut self, spec: NodeSpec) -> Result<NodeId> {
        let ports_spec = spec.ports.clone();
        let id = self.node_service.create(
            self.node_facade_view.specs,
            self.node_facade_view.nodes,
            spec,
        )?;
        Self::initialize_ports(self.node_facade_view.ports, id, &ports_spec)?;
        //@todo should parameters also be initialized? Better if user provides already checked value
        Ok(id)
    }

    pub fn remove(&mut self, id: NodeId) -> Result<()> {
        self.node_service.remove::<NRF>(self.node_facade_view, id)?;
        self.edge_removal_service_api.on_removal(id)?;
        self.disconnect_ports(id)?;
        Ok(())
    }

    fn initialize_ports(
        repo: &mut impl PortStateRepositoryConcept,
        id: NodeId,
        ports_spec: &PortsSpec,
    ) -> Result<()> {
        for port_id in ports_spec.ids() {
            repo.create(id, *port_id)?;
        }
        Ok(())
    }

    fn disconnect_ports(&mut self, id: NodeId) -> Result<()> {
        let spec_service_api =
            SpecServiceApi::new(self.node_facade_view.specs, self.node_facade_view.nodes);
        let channel_service_api =
            ChannelServiceApi::new(self.channel_facade.view_mut(), self.channel_service);
        let mut port_connection_service_api = PortConnectionServiceApi::new(
            self.node_facade_view.ports,
            spec_service_api,
            channel_service_api,
        );

        port_connection_service_api.disconnect_all(id)
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
        self.repo.ids()
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

    //@todo move to NodeLoadApi
    pub fn load(&mut self, id: NodeId, params: Parameters) -> Result<()> {
        //@todo validate against schema
        self.repo.insert(id, params)
    }

    //@todo add API to set parameters, also validate against schema here
}

pub struct PortConnectionInput {
    node: NodeId,
    port: NodePortId,
    channel: ChannelId,
}

pub struct PortConnectionServiceApi<'a, PR, SR, NR, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    repo: &'a mut PR,
    spec_service_api: SpecServiceApi<'a, SR, NR>,
    channel_service_api: ChannelServiceApi<'a, CRF>,
}

impl<'a, PR, SR, NR, CRF> PortConnectionServiceApi<'a, PR, SR, NR, CRF>
where
    PR: PortStateRepositoryConcept,
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
    NR: NodeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    pub(super) fn new(
        repo: &'a mut PR,
        spec_service_api: SpecServiceApi<'a, SR, NR>,
        channel_service_api: ChannelServiceApi<'a, CRF>,
    ) -> Self {
        Self {
            repo,
            spec_service_api,
            channel_service_api,
        }
    }

    pub fn state(&self, node_id: NodeId, port_id: NodePortId) -> Result<&NodePortConnection> {
        self.repo.state(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) state")
        })
    }

    pub fn connect(&mut self, input: PortConnectionInput) -> Result<()> {
        let spec = self.spec_service_api.ports(input.node)?.spec(input.port)?;
        let ctx = ConnectionContext {
            channel: input.channel,
            node: input.node,
            spec,
        };
        self.channel_service_api.connect(ctx)?;
        self.state_mut(input.node, input.port)?
            .connect(input.channel)
    }

    pub fn disconnect(&mut self, input: PortConnectionInput) -> Result<()> {
        let spec = self.spec_service_api.ports(input.node)?.spec(input.port)?;
        self.channel_service_api
            .disconnect(input.channel, spec.kind)?;
        self.state_mut(input.node, input.port)?
            .disconnect(input.channel)
    }

    pub fn disconnect_all(&mut self, id: NodeId) -> Result<()> {
        let ports_spec = self.spec_service_api.ports(id)?.clone();
        for port_id in ports_spec.ids() {
            let channels = self.state_mut(id, *port_id)?.disconnect_all();
            for channel in channels {
                self.channel_service_api
                    .disconnect(channel, ports_spec.spec(*port_id)?.kind)?;
            }
        }
        Ok(())
    }

    //@todo move to NodeLoadApi
    fn load(
        &mut self,
        id: NodeId,
        records: impl IntoIterator<Item = NodePortRecord>,
    ) -> Result<()> {
        for record in records {
            //@todo validate that conn is valid
            self.repo.load(id, record.id, record.conn)?;
        }
        Ok(())
    }

    fn state_mut(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
    ) -> Result<&mut NodePortConnection> {
        self.repo.state_mut(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) state")
        })
    }
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
        self.validate_creation(kind)?;
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

    fn validate_creation(&self, kind: NodeKind) -> Result<()> {
        if let NodeKind::Root = kind
            && let Some(root) = self.node_cache.get(&kind)
            && !root.is_empty()
        {
            bail!("attempted to create multiple roots");
        }
        Ok(())
    }

    fn load_node(
        &mut self,
        spec_repo: &impl SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
        node_repo: &mut impl NodeRepositoryConcept,
        id: NodeId,
        spec_id: NodeSpecId,
    ) -> Result<()> {
        let spec_api = SpecServiceApi {
            spec_repo,
            node_repo,
        };
        let kind = spec_api.kind_by_spec_id(spec_id)?;
        self.validate_creation(kind)?;
        node_repo.load(id, spec_id)?;
        self.node_cache.entry(kind).or_default().insert(id);
        Ok(())
    }

    fn load_spec(
        &mut self,
        spec_repo: &mut impl SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
        record: NodeSpecRecord,
    ) -> Result<()> {
        match self.spec_cache.get(&record.spec) {
            Some(id) => {
                warn!("spec {id} was already loaded");
            }
            None => {
                let NodeSpecRecord { id, spec } = record;
                spec_repo.load(id, spec.clone())?;
                self.spec_cache.insert(spec, id);
            }
        }
        Ok(())
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
        let spec = SpecServiceApi::spec_by_node_id(view.specs, view.nodes, id)?;
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

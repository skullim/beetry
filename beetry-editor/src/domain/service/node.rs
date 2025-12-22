use crate::domain::{
    repository::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, NodeRepositoryConcept,
        NodeRepositoryFacadeConcept, NodeRepositoryFacadeView, NodeRepositoryFacadeViewMut,
        ParamValueRepositoryConcept, PortStateRepositoryConcept, SpecRepositoryConcept,
        UiRepositoryConcept,
    },
    service::{
        channel::{ChannelBorrowMutApi, ChannelService, ConnectionContext},
        edge::{self, EdgeService, OnNodeRemovalServiceApi},
    },
};
use anyhow::{Context, Result, anyhow, bail};
use beetry_plugin_types::node::NodeName;
use beetry_reconstruction_types::parameter::Parameters;
use std::collections::{HashMap, HashSet};
use tracing::warn;

use beetry_editor_types::{
    ChannelId, NodeId, NodeKind, NodePortConnection, NodePortId, NodePortState, NodePosition,
    NodeRecord, NodeSpec, NodeSpecId, NodeSpecKey, NodeUiData, ParameterValue, PortsSpec,
};

pub struct NodeBorrowApi<'a, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    facade_view: NodeRepositoryFacadeView<'a, NRF>,
    node_service: &'a NodeService,
}
impl<'a, NRF> NodeBorrowApi<'a, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    pub(super) fn new(
        facade_view: NodeRepositoryFacadeView<'a, NRF>,
        node_service: &'a NodeService,
    ) -> Self {
        Self {
            facade_view,
            node_service,
        }
    }

    pub fn spec(&self) -> SpecApi<'_, NRF::SpecRepo, NRF::NodeRepo> {
        SpecApi {
            spec_repo: self.facade_view.specs,
            node_repo: self.facade_view.nodes,
        }
    }

    pub fn tracker(&self) -> TrackerApi<'_, NRF::NodeRepo> {
        TrackerApi {
            service: self.node_service,
            repo: self.facade_view.nodes,
        }
    }

    pub fn port_state(&self) -> PortStateApi<'_, NRF::PortStateRepo> {
        PortStateApi::new(self.facade_view.ports)
    }

    pub fn parameter(&self) -> ParameterValueBorrowApi<'_, NRF::ParamValuesRepo> {
        ParameterValueBorrowApi {
            repo: self.facade_view.parameters,
        }
    }
}
/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct NodeBorrowMutApi<'a, NRF, ER, CRF>
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

impl<'a, NRF, ER, CRF> NodeBorrowMutApi<'a, NRF, ER, CRF>
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

    pub fn port_connection(
        &mut self,
    ) -> PortConnectionApi<'_, NRF::PortStateRepo, NRF::SpecRepo, NRF::NodeRepo, CRF> {
        PortConnectionApi {
            repo: self.facade_view.ports,
            spec_service_api: SpecApi {
                spec_repo: self.facade_view.specs,
                node_repo: self.facade_view.nodes,
            },
            channel_service_api: ChannelBorrowMutApi::new(
                self.channel_facade.view_mut(),
                self.channel_service,
            ),
        }
    }

    pub fn parameters(&mut self) -> ParameterValueBorrowMutApi<'_, NRF::ParamValuesRepo> {
        ParameterValueBorrowMutApi {
            repo: self.facade_view.parameters,
        }
    }
}

pub struct SpecApi<'a, SR, NR> {
    spec_repo: &'a SR,
    node_repo: &'a NR,
}

impl<'a, SR, NR> SpecApi<'a, SR, NR>
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
        Ok(Self::spec_by_node_id(self.spec_repo, self.node_repo, id)?.name())
    }

    pub fn kind(&self, id: NodeId) -> Result<NodeKind> {
        Ok(Self::spec_by_node_id(self.spec_repo, self.node_repo, id)?.kind())
    }

    pub fn ports(&self, id: NodeId) -> Result<&PortsSpec> {
        Ok(Self::spec_by_node_id(self.spec_repo, self.node_repo, id)?.ports())
    }

    fn kind_by_spec_id(&self, spec_id: NodeSpecId) -> Result<NodeKind> {
        Ok(Self::spec_by_spec_id(self.spec_repo, spec_id)?.kind())
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

pub(super) struct LoadNodeApi<'a, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    node_service: &'a mut NodeService,
    node_facade_view: NodeRepositoryFacadeViewMut<'a, NRF>,
}

impl<'a, NRF> LoadNodeApi<'a, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    pub(super) fn new(
        node_service: &'a mut NodeService,
        node_facade_view: NodeRepositoryFacadeViewMut<'a, NRF>,
    ) -> Self {
        Self {
            node_service,
            node_facade_view,
        }
    }

    pub(super) fn load_node(
        &mut self,
        node: NodeRecord,
        param_value: Option<ParameterValue>,
        port_state: Option<NodePortState>,
    ) -> Result<()> {
        self.node_service.load_node(
            self.node_facade_view.specs,
            self.node_facade_view.nodes,
            node.id,
            node.value.spec_id(),
        )?;
        if let Some(state) = port_state {
            self.load_ports(node.id, state)?;
        }

        if let Some(value) = param_value {
            let mut params_service_api = ParameterValueBorrowMutApi {
                repo: self.node_facade_view.parameters,
            };
            params_service_api.load(node.id, value)?;
        }
        Ok(())
    }

    fn load_ports(&mut self, id: NodeId, state: NodePortState) -> Result<()> {
        //@todo should there be a validation that given channels exist?
        for (port_id, conn) in state.conns {
            self.node_facade_view.ports.create(id, port_id, conn)?;
        }
        Ok(())
    }

    pub(super) fn load_spec(&mut self, id: NodeSpecId, spec: NodeSpec) -> Result<()> {
        self.node_service
            .load_spec(self.node_facade_view.specs, id, spec)
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
        let ports_spec = spec.ports().clone();
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
            repo.create(id, *port_id, NodePortConnection::default())?;
        }
        Ok(())
    }

    fn disconnect_ports(&mut self, id: NodeId) -> Result<()> {
        let spec_service_api =
            SpecApi::new(self.node_facade_view.specs, self.node_facade_view.nodes);
        let channel_service_api =
            ChannelBorrowMutApi::new(self.channel_facade.view_mut(), self.channel_service);
        let mut port_connection_service_api = PortConnectionApi::new(
            self.node_facade_view.ports,
            spec_service_api,
            channel_service_api,
        );

        port_connection_service_api.disconnect_all(id)
    }
}

pub struct TrackerApi<'a, NR> {
    service: &'a NodeService,
    repo: &'a NR,
}

impl<'a, NR> TrackerApi<'a, NR>
where
    NR: NodeRepositoryConcept,
{
    pub(super) fn new(service: &'a NodeService, repo: &'a NR) -> Self {
        Self { service, repo }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &NodeId> {
        self.repo.ids()
    }

    pub fn leaf_nodes(&self) -> impl Iterator<Item = &NodeId> {
        self.nodes_by_kind(NodeKind::Action)
            .chain(self.nodes_by_kind(NodeKind::Condition))
    }

    pub fn spec_id(&self, id: NodeId) -> Result<NodeSpecId> {
        self.repo
            .spec_id(&id)
            .copied()
            .ok_or_else(|| anyhow!("no mapping between node id {id} and spec id exists"))
    }

    pub fn root_id(&self) -> Result<NodeId> {
        self.nodes_by_kind(NodeKind::Root)
            .next()
            .copied()
            .ok_or_else(|| anyhow!("no root found in the tree"))
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

pub struct ParameterValueBorrowApi<'a, PVR> {
    repo: &'a PVR,
}

impl<'a, PVR> ParameterValueBorrowApi<'a, PVR>
where
    PVR: ParamValueRepositoryConcept,
{
    pub fn parameters(&self, id: NodeId) -> Result<&Parameters> {
        self.repo
            .value(id)
            .ok_or_else(|| anyhow!("failed to obtain parameters for node {id}"))
    }
}

pub struct ParameterValueBorrowMutApi<'a, PVR> {
    repo: &'a mut PVR,
}

impl<'a, PVR> ParameterValueBorrowMutApi<'a, PVR>
where
    PVR: ParamValueRepositoryConcept,
{
    //@todo move to LoadNodeApi
    pub fn load(&mut self, id: NodeId, value: ParameterValue) -> Result<()> {
        self.repo.create(id, value.params)
    }

    //@todo add API to set parameters, also validate against schema here
}

pub struct PortConnectionInput {
    node: NodeId,
    port: NodePortId,
    channel: ChannelId,
}

pub struct PortStateApi<'a, PR> {
    repo: &'a PR,
}

impl<'a, PR> PortStateApi<'a, PR>
where
    PR: PortStateRepositoryConcept,
{
    pub(super) fn new(repo: &'a PR) -> Self {
        Self { repo }
    }

    pub fn port_iter(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = (&NodePortId, &NodePortConnection)> {
        self.repo.port_iter(node_id)
    }

    pub fn state(&self, node_id: NodeId, port_id: NodePortId) -> Result<&NodePortConnection> {
        self.repo.state(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) state")
        })
    }
}

pub struct PortConnectionApi<'a, PR, SR, NR, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    repo: &'a mut PR,
    spec_service_api: SpecApi<'a, SR, NR>,
    channel_service_api: ChannelBorrowMutApi<'a, CRF>,
}

impl<'a, PR, SR, NR, CRF> PortConnectionApi<'a, PR, SR, NR, CRF>
where
    PR: PortStateRepositoryConcept,
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
    NR: NodeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    pub(super) fn new(
        repo: &'a mut PR,
        spec_service_api: SpecApi<'a, SR, NR>,
        channel_service_api: ChannelBorrowMutApi<'a, CRF>,
    ) -> Self {
        Self {
            repo,
            spec_service_api,
            channel_service_api,
        }
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

#[derive(Debug, Default)]
pub(super) struct NodeService {
    spec_cache: HashMap<NodeSpecKey, NodeSpecId>,
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
        let kind = spec.kind();
        self.validate_creation(kind)?;
        let spec_id = match self.spec_cache.get(spec.key()) {
            Some(id) => *id,
            None => {
                let spec_id = spec_repo.create(spec.clone())?;
                self.spec_cache.insert(spec.key, spec_id);
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
        let spec_api = SpecApi {
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
        id: NodeSpecId,
        spec: NodeSpec,
    ) -> Result<()> {
        match self.spec_cache.get(spec.key()) {
            Some(id) => {
                warn!("spec {id} is already loaded");
            }
            None => {
                spec_repo.load(id, spec.clone())?;
                self.spec_cache.insert(spec.key, id);
            }
        }
        Ok(())
    }

    pub(super) fn positions_by_kind<'a>(
        &self,
        repo: &'a impl UiRepositoryConcept<Id = NodeId, Data = NodeUiData>,
        kind: NodeKind,
    ) -> impl Iterator<Item = &'a NodePosition> {
        let position_ids = self
            .node_cache
            .get(&kind)
            .into_iter()
            .flat_map(|i| i.iter().copied());
        position_ids
            .flat_map(|id| repo.data(id))
            .map(|data| &data.position)
    }

    fn remove<NRF>(
        &mut self,
        view: &mut NodeRepositoryFacadeViewMut<'_, NRF>,
        id: NodeId,
    ) -> Result<()>
    where
        NRF: NodeRepositoryFacadeConcept,
    {
        let spec = SpecApi::spec_by_node_id(view.specs, view.nodes, id)?;
        self.node_cache
            .get_mut(&spec.kind())
            .map(|nodes| nodes.remove(&id));

        view.parameters.remove(id)
    }

    pub(super) fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId> {
        self.node_cache
            .get(&kind)
            .into_iter()
            .flat_map(|nodes| nodes.iter())
    }
}

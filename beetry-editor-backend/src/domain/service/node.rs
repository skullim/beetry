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
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};

use beetry_editor_types::{
    id::{ChannelId, NodeId, NodePortId, NodeSpecId},
    output::{
        node::{Parameters, PortConnectionState},
        ui::{NodePosition, NodeUiData},
    },
    persistence::{NodeRecord, ParameterValue, PortConnectionCollection},
    spec::node::{NodeKind, NodeName, NodeSpec, NodeSpecKey, ParamsSpec, PortsSpec},
};
use mitsein::iter1::FromIterator1;

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
        &'a mut self,
        ports_spec: &'a PortsSpec,
    ) -> PortConnectionApi<'a, NRF::PortStateRepo, CRF> {
        PortConnectionApi {
            repo: self.facade_view.ports,
            ports_spec,
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

    // borrow mut has also access to borrow api
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
        Self::spec_by_node_id(self.spec_repo, self.node_repo, id)?
            .ports()
            .as_ref()
            .ok_or_else(|| anyhow!("expected port specification for node {id}"))
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

    pub fn spec_by_node_id_pub(&self, id: NodeId) -> Result<&NodeSpec> {
        Self::spec_by_node_id(self.spec_repo, self.node_repo, id)
    }

    fn spec_by_node_id<'s>(spec_repo: &'s SR, node_repo: &NR, id: NodeId) -> Result<&'s NodeSpec> {
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

/// API used to load the given record from the storage. It is assumed that valid entities are loaded, i.e.
/// entities that have been created only using the provided interface. Therefore no further validation is implemented (as opposed to
/// the interface that is used to create the entities).
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
        port_state: Option<PortConnectionCollection>,
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
            self.load_parameters(node.id, value)?;
        }
        Ok(())
    }

    fn load_parameters(&mut self, id: NodeId, value: ParameterValue) -> Result<()> {
        self.node_facade_view.parameters.create(id, value.params)
    }

    fn load_ports(&mut self, id: NodeId, state: PortConnectionCollection) -> Result<()> {
        for conn_record in state.conns {
            self.node_facade_view
                .ports
                .insert(id, conn_record.port_id, conn_record.conn)?;
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
    pub fn create(&mut self, spec: &NodeSpec) -> Result<NodeId> {
        let id = self.node_service.create(
            self.node_facade_view.specs,
            self.node_facade_view.nodes,
            spec,
        )?;
        Ok(id)
    }

    pub fn remove(&mut self, spec: &NodeSpec, id: NodeId) -> Result<()> {
        self.node_service
            .remove::<NRF>(self.node_facade_view, spec, id)?;
        self.edge_removal_service_api.on_removal(id)?;
        if let Some(ports_spec) = spec.ports() {
            let channel_service_api =
                ChannelBorrowMutApi::new(self.channel_facade.view_mut(), self.channel_service);
            let mut port_connection_service_api = PortConnectionApi::new(
                self.node_facade_view.ports,
                ports_spec,
                channel_service_api,
            );

            port_connection_service_api.disconnect_all(id)?;
        }
        if spec.params().is_some() {
            self.node_facade_view.parameters.remove(id);
        }
        Ok(())
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
        self.nodes_by_kind(NodeKind::action())
            .chain(self.nodes_by_kind(NodeKind::condition()))
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
    pub fn create(&mut self, id: NodeId, value: ParameterValue) -> Result<()> {
        //@todo validate against schema here
        self.repo.create(id, value.params)
    }
}

pub struct PortConnectionInput {
    node: NodeId,
    port: NodePortId,
    channel: ChannelId,
}

impl PortConnectionInput {
    pub fn new(node: NodeId, port: NodePortId, channel: ChannelId) -> Self {
        Self {
            node,
            port,
            channel,
        }
    }
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

    pub fn state(&self, node_id: NodeId, port_id: NodePortId) -> Result<&PortConnectionState> {
        self.repo.state(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) state")
        })
    }

    pub fn node_conns(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = (&NodePortId, &PortConnectionState)> {
        self.repo.node_conns(node_id)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            &NodeId,
            impl Iterator<Item = (&NodePortId, &PortConnectionState)>,
        ),
    > {
        self.repo.iter()
    }
}

pub struct PortConnectionApi<'a, PR, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    repo: &'a mut PR,
    ports_spec: &'a PortsSpec,
    channel_service_api: ChannelBorrowMutApi<'a, CRF>,
}

impl<'a, PR, CRF> PortConnectionApi<'a, PR, CRF>
where
    PR: PortStateRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    pub(super) fn new(
        repo: &'a mut PR,
        ports_spec: &'a PortsSpec,
        channel_service_api: ChannelBorrowMutApi<'a, CRF>,
    ) -> Self {
        Self {
            repo,
            ports_spec,
            channel_service_api,
        }
    }

    pub fn connect(&mut self, input: PortConnectionInput) -> Result<()> {
        let spec = self.ports_spec.spec(input.port)?;
        let ctx = ConnectionContext {
            channel: input.channel,
            node: input.node,
            spec,
        };
        self.channel_service_api.connect(ctx)?;
        if let Some(conn) = self.state_mut(input.node, input.port) {
            conn.connect(input.channel)?;
        } else {
            self.repo.insert(
                input.node,
                input.port,
                PortConnectionState::Internal(<_>::try_from_iter(std::iter::once(input.channel))?),
            )?;
        }
        Ok(())
    }

    pub fn set_external(&mut self, id: NodeId, port: NodePortId) -> Result<()> {
        self.disconnect_all(id)?;
        self.repo.insert(id, port, PortConnectionState::External)
    }

    pub fn disconnect(&mut self, input: PortConnectionInput) -> Result<()> {
        let spec = self.ports_spec.spec(input.port)?;
        self.channel_service_api
            .disconnect(input.channel, spec.kind)?;
        if let Some(conn) = self.repo.remove(input.node, input.port)
            && let Some(still_valid_conn) = conn.disconnect(input.channel)?
        {
            self.repo.insert(input.node, input.port, still_valid_conn)?;
        }

        Ok(())
    }

    pub fn disconnect_all(&mut self, id: NodeId) -> Result<()> {
        for port_id in self.ports_spec.ids() {
            if let Some(conn) = self.repo.remove(id, *port_id) {
                let channels = conn.disconnect_all();
                for channel in channels {
                    self.channel_service_api
                        .disconnect(channel, self.ports_spec.spec(*port_id)?.kind)?;
                }
            }
        }
        Ok(())
    }

    fn state_mut(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
    ) -> Option<&mut PortConnectionState> {
        self.repo.state_mut(node_id, port_id)
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

    // @todo might consider Cow for NodeSpec at some point, reference semantics better than value
    // as there might be multiple nodes created of the same type
    fn create(
        &mut self,
        spec_repo: &mut impl SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
        node_repo: &mut impl NodeRepositoryConcept,
        spec: &NodeSpec,
    ) -> Result<NodeId> {
        let kind = spec.kind();
        self.validate_creation(kind)?;
        let spec_id = match self.spec_cache.get(spec.key()) {
            Some(id) => *id,
            None => {
                debug!("inserting new spec into spec repo");
                let spec_id = spec_repo.create(spec.clone())?;
                self.spec_cache.insert(spec.key.clone(), spec_id);
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

    //@todo this should be moved into Ui service
    pub(super) fn positions_by_kind<'a>(
        &self,
        repo: &'a impl UiRepositoryConcept<Id = NodeId, Data = NodeUiData>,
        kind: NodeKind,
    ) -> impl Iterator<Item = (NodeId, &'a NodePosition)> {
        let position_ids = self
            .node_cache
            .get(&kind)
            .into_iter()
            .flat_map(|i| i.iter().copied());
        position_ids.flat_map(|id| repo.data(id).map(|data| (id, &data.position)))
    }

    fn remove<NRF>(
        &mut self,
        view: &mut NodeRepositoryFacadeViewMut<'_, NRF>,
        spec: &NodeSpec,
        id: NodeId,
    ) -> Result<()>
    where
        NRF: NodeRepositoryFacadeConcept,
    {
        self.node_cache
            .get_mut(&spec.kind())
            .map(|nodes| nodes.remove(&id));
        view.nodes.remove(id);
        Ok(())
    }

    pub(super) fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId> {
        self.node_cache
            .get(&kind)
            .into_iter()
            .flat_map(|nodes| nodes.iter())
    }
}

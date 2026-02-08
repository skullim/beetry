use crate::domain::{
    repository::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, NodeRepositoryConcept,
        NodeRepositoryFacadeConcept, NodeRepositoryFacadeView, NodeRepositoryFacadeViewMut,
        ParamValueRepositoryConcept, PortStateRepositoryConcept, SpecRepositoryConcept,
    },
    service::{
        channel::{ChannelService, ChannelViewMut, ConnectionContext},
        edge::{self, EdgeService, OnNodeRemovalServiceApi},
    },
};
use anyhow::{Context, Result, anyhow, bail};
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};

use beetry_editor_types::{
    id::{ChannelId, NodeId, NodePortId, NodeSpecId},
    output::node::{ParameterValue, Parameters, PortConnectionState},
    persistence::{NodeRecord, ParameterValues, PortConnectionCollection},
    spec::node::{
        FieldTypeSpec, NodeKind, NodeName, NodePortKind, NodeSpec, NodeSpecKey, ParamsSpec,
        PortsSpec,
    },
};
use mitsein::iter1::FromIterator1;

pub struct NodeView<'a, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    facade_view: NodeRepositoryFacadeView<'a, NRF>,
    node_service: &'a NodeService,
}
impl<'a, NRF> NodeView<'a, NRF>
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

    pub fn spec(&self) -> SpecView<'_, NRF::SpecRepo, NRF::NodeRepo> {
        SpecView {
            spec_repo: self.facade_view.specs,
            node_repo: self.facade_view.nodes,
        }
    }

    pub fn tracker(&self) -> TrackerView<'_, NRF::NodeRepo> {
        TrackerView {
            service: self.node_service,
            repo: self.facade_view.nodes,
        }
    }

    pub fn port_state(
        &self,
    ) -> PortStateView<'_, NRF::NodeRepo, NRF::SpecRepo, NRF::PortStateRepo> {
        PortStateView::new(
            self.facade_view.nodes,
            self.facade_view.specs,
            self.facade_view.ports,
        )
    }

    pub fn into_port_state(
        self,
    ) -> PortStateView<'a, NRF::NodeRepo, NRF::SpecRepo, NRF::PortStateRepo> {
        PortStateView::new(
            self.facade_view.nodes,
            self.facade_view.specs,
            self.facade_view.ports,
        )
    }

    pub fn parameter(&self) -> ParameterValueView<'_, NRF::ParamValuesRepo> {
        ParameterValueView {
            repo: self.facade_view.parameters,
        }
    }
}
/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct NodeViewMut<'a, NRF, ER, CRF>
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

impl<'a, NRF, ER, CRF> NodeViewMut<'a, NRF, ER, CRF>
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

    pub(crate) fn lifecycle(&'a mut self) -> NodeLifecycleView<'a, NRF, CRF, ER> {
        NodeLifecycleView {
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

    pub fn port_state(
        &'a mut self,
    ) -> PortStateViewMut<'a, NRF::NodeRepo, NRF::SpecRepo, NRF::PortStateRepo, CRF> {
        PortStateViewMut::new(
            self.facade_view.nodes,
            self.facade_view.specs,
            self.facade_view.ports,
            self.channel_facade,
            self.channel_service,
        )
    }
}

pub struct SpecView<'a, SR, NR> {
    spec_repo: &'a SR,
    node_repo: &'a NR,
}

impl<'a, SR, NR> SpecView<'a, SR, NR>
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

pub trait SpecBySpecIdQueryView {
    fn spec(&self, id: NodeSpecId) -> Result<&NodeSpec>;
    fn name(&self, id: NodeSpecId) -> Result<&NodeName>;
    fn kind(&self, id: NodeSpecId) -> Result<NodeKind>;
}

pub struct SpecBySpecIdQuery<'a, SR> {
    repo: &'a SR,
}

impl<'a, SR> SpecBySpecIdQuery<'a, SR> {
    pub(super) fn new(repo: &'a SR) -> Self {
        Self { repo }
    }
}

impl<'a, SR> SpecBySpecIdQueryView for SpecBySpecIdQuery<'a, SR>
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

pub struct SpecByNodeIdQuery<'a, SR, NR> {
    spec_query: SpecBySpecIdQuery<'a, SR>,
    node_repo: &'a NR,
}

impl<'a, SR, NR> SpecByNodeIdQuery<'a, SR, NR>
where
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
    NR: NodeRepositoryConcept,
{
    pub fn new(spec_query: SpecBySpecIdQuery<'a, SR>, node_repo: &'a NR) -> Self {
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

pub trait SpecByNodeIdQueryView {
    fn spec(&self, id: NodeId) -> Result<&NodeSpec>;
    fn name(&self, id: NodeId) -> Result<&NodeName>;
    fn kind(&self, id: NodeId) -> Result<NodeKind>;
    fn ports(&self, id: NodeId) -> Result<&PortsSpec>;
    fn params(&self, id: NodeId) -> Result<&ParamsSpec>;
}

impl<'a, SR, NR> SpecByNodeIdQueryView for SpecByNodeIdQuery<'a, SR, NR>
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

/// API used to load the given record from the storage. It is assumed that valid entities are loaded, i.e.
/// entities that have been created only using the provided interface. Therefore no further validation is implemented (as opposed to
/// the interface that is used to create the entities).
pub(super) struct LoadNodeView<'a, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    node_service: &'a mut NodeService,
    node_facade_view: NodeRepositoryFacadeViewMut<'a, NRF>,
}

impl<'a, NRF> LoadNodeView<'a, NRF>
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
        param_value: Option<ParameterValues>,
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
            self.load_parameters(node.id, value);
        }
        Ok(())
    }

    fn load_parameters(&mut self, id: NodeId, value: ParameterValues) {
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

pub(crate) struct NodeLifecycleView<'a, NRF, CRF, ER>
where
    NRF: NodeRepositoryFacadeConcept,
{
    node_service: &'a mut NodeService,
    channel_service: &'a mut ChannelService,
    node_facade_view: &'a mut NodeRepositoryFacadeViewMut<'a, NRF>,
    channel_facade: &'a mut CRF,
    edge_removal_service_api: edge::OnNodeRemovalServiceApi<'a, ER>,
}

impl<'a, NRF, CRF, ER> NodeLifecycleView<'a, NRF, CRF, ER>
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
        self.node_service.remove(spec, id)?;
        self.node_facade_view.nodes.remove(id);
        self.edge_removal_service_api.on_removal(id)?;
        if let Some(ports_spec) = spec.ports() {
            let channel_service_api =
                ChannelViewMut::new(self.channel_facade.view_mut(), self.channel_service);
            let mut port_connection_service_api = PortConnectionView::new(
                self.node_facade_view.ports,
                ports_spec,
                channel_service_api,
            );

            port_connection_service_api.disconnect_all_ports(id)?;
        }
        if spec.params().is_some() {
            self.node_facade_view.parameters.remove(id);
        }
        Ok(())
    }
}

pub struct TrackerView<'a, NR> {
    service: &'a NodeService,
    repo: &'a NR,
}

impl<'a, NR> TrackerView<'a, NR>
where
    NR: NodeRepositoryConcept,
{
    pub(super) fn new(service: &'a NodeService, repo: &'a NR) -> Self {
        Self { service, repo }
    }

    pub(super) fn ensure_exists(&self, id: NodeId) -> Result<()> {
        if !self.repo.contains(&id) {
            bail!("node {id} does not exist");
        }
        Ok(())
    }
}

pub trait NodeTrackerQueryView {
    fn nodes(&self) -> impl Iterator<Item = &NodeId>;
    fn leaf_nodes(&self) -> impl Iterator<Item = &NodeId>;
    fn spec_id(&self, id: NodeId) -> Result<NodeSpecId>;
    fn root_id(&self) -> Result<NodeId>;
    fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId>;
}

impl<NR> NodeTrackerQueryView for TrackerView<'_, NR>
where
    NR: NodeRepositoryConcept,
{
    fn nodes(&self) -> impl Iterator<Item = &NodeId> {
        self.repo.ids()
    }

    fn leaf_nodes(&self) -> impl Iterator<Item = &NodeId> {
        self.nodes_by_kind(NodeKind::action())
            .chain(self.nodes_by_kind(NodeKind::condition()))
    }

    fn spec_id(&self, id: NodeId) -> Result<NodeSpecId> {
        self.repo
            .spec_id(&id)
            .copied()
            .ok_or_else(|| anyhow!("no mapping between node id {id} and spec id exists"))
    }

    fn root_id(&self) -> Result<NodeId> {
        self.nodes_by_kind(NodeKind::Root)
            .next()
            .copied()
            .ok_or_else(|| anyhow!("no root found in the tree"))
    }

    fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId> {
        self.service.nodes_by_kind(kind)
    }
}

pub struct ParameterValueView<'a, PVR> {
    repo: &'a PVR,
}

impl<'a, PVR> ParameterValueView<'a, PVR>
where
    PVR: ParamValueRepositoryConcept,
{
    pub fn parameters(&self, id: NodeId) -> Result<&Parameters> {
        self.repo
            .params(id)
            .ok_or_else(|| anyhow!("failed to obtain parameters for node {id}"))
    }
}

pub struct ParameterValueParser;

impl ParameterValueParser {
    pub fn parse(type_spec: &FieldTypeSpec, raw_value: String) -> Result<ParameterValue> {
        Ok(match &type_spec {
            FieldTypeSpec::Bool(meta) => {
                let parsed = raw_value.parse()?;
                meta.validate(&parsed)?;
                ParameterValue::Bool(parsed)
            }
            FieldTypeSpec::F64(meta) => {
                let parsed = raw_value.parse()?;
                meta.validate(&parsed)?;
                ParameterValue::F64(parsed)
            }
            FieldTypeSpec::I64(meta) => {
                let parsed = raw_value.parse()?;
                meta.validate(&parsed)?;
                ParameterValue::I64(parsed)
            }
            FieldTypeSpec::U64(meta) => {
                let parsed = raw_value.parse()?;
                meta.validate(&parsed)?;
                ParameterValue::U64(parsed)
            }
            FieldTypeSpec::String(meta) => {
                meta.validate(&raw_value)?;
                ParameterValue::String(raw_value)
            }
        })
    }
}

pub struct ParameterValueViewMut<'a, PVR> {
    repo: &'a mut PVR,
}

pub trait ParameterValueMut {
    fn create(&mut self, id: NodeId, params: Parameters);
}

impl<'a, PVR> ParameterValueViewMut<'a, PVR>
where
    PVR: ParamValueRepositoryConcept,
{
    pub(super) fn new(repo: &'a mut PVR) -> Self {
        Self { repo }
    }

    pub fn create(&mut self, id: NodeId, params: Parameters) {
        self.repo.create(id, params);
    }
}

impl<PVR> ParameterValueMut for ParameterValueViewMut<'_, PVR>
where
    PVR: ParamValueRepositoryConcept,
{
    fn create(&mut self, id: NodeId, params: Parameters) {
        ParameterValueViewMut::create(self, id, params)
    }
}

pub struct PortStateView<'a, NR, SR, PR> {
    node_repo: &'a NR,
    spec_repo: &'a SR,
    port_repo: &'a PR,
}

impl<'a, NR, SR, PR> PortStateView<'a, NR, SR, PR>
where
    NR: NodeRepositoryConcept,
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
    PR: PortStateRepositoryConcept,
{
    pub(super) fn new(node_repo: &'a NR, spec_repo: &'a SR, port_repo: &'a PR) -> Self {
        Self {
            node_repo,
            spec_repo,
            port_repo,
        }
    }

    pub fn internal_connections(
        self,
    ) -> impl Iterator<Item = (&'a NodeId, &'a NodePortId, &'a ChannelId)> {
        self.port_repo
            .iter()
            .flat_map(|(node_id, port_iter): (&'a NodeId, _)| {
                port_iter
                    .filter_map(|(port_id, state): (&'a NodePortId, &PortConnectionState)| {
                        match state {
                            PortConnectionState::Internal(conns) => Some((port_id, conns)),
                            PortConnectionState::External => None,
                        }
                    })
                    .flat_map(move |(port_id, conns)| {
                        conns
                            .iter()
                            .map(move |channel_id| (node_id, port_id, channel_id))
                    })
            })
    }

    pub fn connection_views(self) -> impl Iterator<Item = Result<PortConnectionDataView<'a>>> {
        PortConnectionViewIter::new(self.node_repo, self.spec_repo, self.internal_connections())
    }

    pub fn connection_views_by_kind(
        self,
        kind: NodePortKind,
    ) -> impl Iterator<Item = Result<PortConnectionDataView<'a>>> {
        self.connection_views().filter_map(move |res| match res {
            Ok(view) if view.kind == kind => Some(Ok(view)),
            Ok(_) => None,
            Err(err) => Some(Err(err)),
        })
    }
}

pub struct PortStateViewMut<'a, NR, SR, PR, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    node_repo: &'a NR,
    spec_repo: &'a SR,
    port_repo: &'a mut PR,
    channel_facade: &'a mut CRF,
    channel_service: &'a mut ChannelService,
}

impl<'a, NR, SR, PR, CRF> PortStateViewMut<'a, NR, SR, PR, CRF>
where
    NR: NodeRepositoryConcept,
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
    PR: PortStateRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    fn new(
        node_repo: &'a NR,
        spec_repo: &'a SR,
        port_repo: &'a mut PR,
        channel_facade: &'a mut CRF,
        channel_service: &'a mut ChannelService,
    ) -> Self {
        Self {
            node_repo,
            spec_repo,
            port_repo,
            channel_facade,
            channel_service,
        }
    }

    fn port_connection_by_node(
        &mut self,
        node_id: NodeId,
    ) -> Result<PortConnectionView<'_, PR, CRF>> {
        let spec = SpecView::spec_by_node_id(self.spec_repo, self.node_repo, node_id)?;
        let ports_spec = spec
            .ports()
            .as_ref()
            .ok_or_else(|| anyhow!("expected port specification for node {node_id}"))?;

        Ok(PortConnectionView::new(
            self.port_repo,
            ports_spec,
            ChannelViewMut::new(self.channel_facade.view_mut(), self.channel_service),
        ))
    }

    pub fn connect_port(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
        channel_id: ChannelId,
    ) -> Result<()> {
        if PortStateQueryApi::is_external(self, node_id, port_id)? {
            bail!("attempted to connect port that is marked as external");
        }
        let mut port_connection = self.port_connection_by_node(node_id)?;
        port_connection.connect(PortConnectionInput::new(node_id, port_id, channel_id))
    }

    pub fn disconnect_port(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
        channel_id: ChannelId,
    ) -> Result<()> {
        let mut port_connection = self.port_connection_by_node(node_id)?;
        port_connection.disconnect(PortConnectionInput::new(node_id, port_id, channel_id))
    }

    pub fn set_port_external(&'a mut self, node_id: NodeId, port_id: NodePortId) -> Result<()> {
        let mut port_connection = self.port_connection_by_node(node_id)?;
        port_connection.set_external(node_id, port_id)
    }

    pub fn set_port_internal(&'a mut self, node_id: NodeId, port_id: NodePortId) -> Result<()> {
        let mut port_connection = self.port_connection_by_node(node_id)?;
        port_connection.disconnect_port(node_id, port_id)
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

struct PortConnectionViewIter<'a, NR, SR, I> {
    node_repo: &'a NR,
    spec_repo: &'a SR,
    iter: I,
}

impl<'a, NR, SR, I> PortConnectionViewIter<'a, NR, SR, I>
where
    NR: NodeRepositoryConcept + 'a,
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = beetry_editor_types::id::NodeSpecId> + 'a,
    I: Iterator<Item = (&'a NodeId, &'a NodePortId, &'a ChannelId)>,
{
    fn new(node_repo: &'a NR, spec_repo: &'a SR, iter: I) -> Self {
        Self {
            node_repo,
            spec_repo,
            iter,
        }
    }
}

impl<'a, NR, SR, I> Iterator for PortConnectionViewIter<'a, NR, SR, I>
where
    NR: NodeRepositoryConcept + 'a,
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = beetry_editor_types::id::NodeSpecId> + 'a,
    I: Iterator<Item = (&'a NodeId, &'a NodePortId, &'a ChannelId)>,
{
    type Item = Result<PortConnectionDataView<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let (node_id, port_id, channel_id) = self.iter.next()?;
        let spec_id = match self.node_repo.spec_id(node_id).copied() {
            Some(spec_id) => spec_id,
            None => {
                return Some(Err(anyhow!(
                    "no mapping between node id {node_id} and spec id exists"
                )));
            }
        };
        let spec = match self.spec_repo.spec(spec_id) {
            Some(spec) => spec,
            None => return Some(Err(anyhow!("failed to obtain spec {spec_id}"))),
        };
        let ports_spec = match spec.ports().as_ref() {
            Some(ports_spec) => ports_spec,
            None => return Some(Err(anyhow!("node {node_id} has no ports spec"))),
        };
        let port_spec = match ports_spec.spec(*port_id) {
            Ok(port_spec) => port_spec,
            Err(err) => return Some(Err(err)),
        };
        Some(Ok(PortConnectionDataView {
            node_id: *node_id,
            port_id: *port_id,
            channel_id: *channel_id,
            kind: port_spec.kind,
            msg_desc: port_spec.msg_spec.as_str(),
        }))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PortConnectionDataView<'a> {
    pub node_id: NodeId,
    pub port_id: NodePortId,
    pub channel_id: ChannelId,
    pub kind: NodePortKind,
    pub msg_desc: &'a str,
}

pub trait PortStateQueryApi {
    fn state(&self, node_id: NodeId, port_id: NodePortId) -> Result<&PortConnectionState>;

    fn node_conns(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = (&NodePortId, &PortConnectionState)>;

    fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            &NodeId,
            impl Iterator<Item = (&NodePortId, &PortConnectionState)>,
        ),
    >;

    fn is_external(&self, node_id: NodeId, port_id: NodePortId) -> Result<bool> {
        Ok(self
            .node_conns(node_id)
            .find(|(id, _)| **id == port_id)
            .map(|(_, state)| state.is_external())
            .unwrap_or(false))
    }
}

impl<NR, SR, PR> PortStateQueryApi for PortStateView<'_, NR, SR, PR>
where
    NR: NodeRepositoryConcept,
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
    PR: PortStateRepositoryConcept,
{
    fn state(&self, node_id: NodeId, port_id: NodePortId) -> Result<&PortConnectionState> {
        self.port_repo.state(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) state")
        })
    }

    fn node_conns(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = (&NodePortId, &PortConnectionState)> {
        self.port_repo.node_conns(node_id)
    }

    fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            &NodeId,
            impl Iterator<Item = (&NodePortId, &PortConnectionState)>,
        ),
    > {
        self.port_repo.iter()
    }
}

impl<NR, SR, PR, CRF> PortStateQueryApi for PortStateViewMut<'_, NR, SR, PR, CRF>
where
    NR: NodeRepositoryConcept,
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>,
    PR: PortStateRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    fn state(&self, node_id: NodeId, port_id: NodePortId) -> Result<&PortConnectionState> {
        self.port_repo.state(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) state")
        })
    }

    fn node_conns(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = (&NodePortId, &PortConnectionState)> {
        self.port_repo.node_conns(node_id)
    }

    fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            &NodeId,
            impl Iterator<Item = (&NodePortId, &PortConnectionState)>,
        ),
    > {
        self.port_repo.iter()
    }
}

pub struct PortConnectionView<'a, PR, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    repo: &'a mut PR,
    ports_spec: &'a PortsSpec,
    channel_service_api: ChannelViewMut<'a, CRF>,
}

impl<'a, PR, CRF> PortConnectionView<'a, PR, CRF>
where
    PR: PortStateRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    pub(super) fn new(
        repo: &'a mut PR,
        ports_spec: &'a PortsSpec,
        channel_service_api: ChannelViewMut<'a, CRF>,
    ) -> Self {
        Self {
            repo,
            ports_spec,
            channel_service_api,
        }
    }
}

pub trait PortConnectionOps {
    fn connect(&mut self, input: PortConnectionInput) -> Result<()>;
    fn set_external(&mut self, id: NodeId, port: NodePortId) -> Result<()>;
    fn disconnect(&mut self, input: PortConnectionInput) -> Result<()>;
    fn disconnect_all_ports(&mut self, id: NodeId) -> Result<()>;
    fn disconnect_port(&mut self, id: NodeId, port: NodePortId) -> Result<()>;
}

impl<PR, CRF> PortConnectionOps for PortConnectionView<'_, PR, CRF>
where
    PR: PortStateRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    fn connect(&mut self, input: PortConnectionInput) -> Result<()> {
        let spec = self.ports_spec.spec(input.port)?;
        let ctx = ConnectionContext {
            channel: input.channel,
            node: input.node,
            spec,
        };
        self.channel_service_api.connect(ctx)?;
        if let Some(conn) = self.repo.state_mut(input.node, input.port) {
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

    fn set_external(&mut self, id: NodeId, port: NodePortId) -> Result<()> {
        self.disconnect_port(id, port)?;
        self.repo.insert(id, port, PortConnectionState::External)
    }

    fn disconnect(&mut self, input: PortConnectionInput) -> Result<()> {
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

    fn disconnect_all_ports(&mut self, id: NodeId) -> Result<()> {
        for port in self.ports_spec.ids() {
            self.disconnect_port(id, *port)?;
        }
        Ok(())
    }

    fn disconnect_port(&mut self, id: NodeId, port: NodePortId) -> Result<()> {
        if let Some(conn) = self.repo.remove(id, port) {
            let channels = conn.disconnect_all();
            for channel in channels {
                self.channel_service_api
                    .disconnect(channel, self.ports_spec.spec(port)?.kind)?;
            }
        }
        Ok(())
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

    fn remove(&mut self, spec: &NodeSpec, id: NodeId) -> Result<()> {
        self.node_cache
            .get_mut(&spec.kind())
            .map(|nodes| nodes.remove(&id));
        Ok(())
    }

    pub(super) fn nodes_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodeId> {
        self.node_cache
            .get(&kind)
            .into_iter()
            .flat_map(|nodes| nodes.iter())
    }
}

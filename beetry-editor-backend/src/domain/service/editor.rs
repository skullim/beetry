use crate::{
    NodeSpecMap,
    channel::ChannelQueryApi,
    domain::{
        channel::ChannelBorrowApi,
        edge::EdgeBorrowApi,
        export::ExportApi,
        import::ImportApi,
        node::NodeBorrowApi,
        repository::{
            ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, EditorRepository,
            EditorRepositoryView, EditorRepositoryViewMut, NodeRepositoryConcept,
            NodeRepositoryFacadeConcept, NodeRepositoryFacadeView, NodeRepositoryFacadeViewMut,
            PortStateRepositoryConcept, SpecRepositoryConcept, UiRepositoryFacadeConcept,
        },
        service::{
            channel::{ChannelBorrowMutApi, ChannelService},
            edge::{EdgeBorrowMutApi, EdgeService},
            node::{self, NodeBorrowMutApi, NodeService},
        },
    },
    edge::EdgeQueryApi,
    node::{
        NodeTrackerQueryApi, ParameterValueBorrowMutApi, ParameterValueMutApi, PortConnectionApi,
        PortConnectionInput, PortConnectionMutApi, PortConnectionView, PortStateApi,
        PortStateQueryApi, SpecByNodeIdQuery, SpecByNodeIdQueryApi, SpecBySpecIdQuery,
        SpecBySpecIdQueryApi, TrackerApi,
    },
    ui::{
        ChannelUiBorrowApi, ChannelUiBorrowMutApi, ChannelUiQueryApi, NodeUiBorrowApi,
        NodeUiBorrowMutApi, NodeUiQueryApi,
    },
};
use anyhow::{Result, anyhow};
use beetry_editor_types::{
    id::{ChannelId, EdgeId, NodeId, NodePortId},
    output::{
        channel::{ChannelConfig, ChannelData},
        edge::NodeEdge,
        node::PortConnectionState,
        ui::{ChannelUiData, NodeUiData, Point},
    },
    spec::{
        channel::ChannelSpec,
        node::{NodePortKind, NodeSpec, PortsSpec},
    },
};

pub struct EditorService<NRF, ER, CRF, URF> {
    node_service: NodeService,
    edge_service: EdgeService,
    channel_service: ChannelService,
    repo: EditorRepository<NRF, ER, CRF, URF>,
    spec_map: NodeSpecMap,
}

impl<NRF, ER, CRF, URF> EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    pub fn new(spec_map: NodeSpecMap) -> Self {
        Self {
            node_service: NodeService::new(),
            edge_service: EdgeService::new(),
            channel_service: ChannelService::new(),
            repo: EditorRepository::<NRF, ER, CRF, URF>::new(),
            spec_map,
        }
    }

    fn node_api(&self) -> NodeBorrowApi<'_, NRF> {
        let EditorRepositoryView { node, .. } = self.repo.view();
        NodeBorrowApi::new(node.view(), &self.node_service)
    }

    fn node_api_mut(&mut self) -> NodeBorrowMutApi<'_, NRF, ER, CRF> {
        let EditorRepositoryViewMut {
            node,
            edge,
            channel,
            ..
        } = self.repo.view_mut();

        NodeBorrowMutApi::new(
            node.view_mut(),
            &mut self.node_service,
            edge,
            &mut self.edge_service,
            channel,
            &mut self.channel_service,
        )
    }

    fn edge_api_mut(&mut self) -> EdgeBorrowMutApi<'_, ER, NRF> {
        let EditorRepositoryViewMut { node, edge, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeView { nodes, specs, .. } = node.view();
        let tracker_api = node::TrackerApi::new(&self.node_service, nodes);
        let spec_api = node::SpecApi::new(specs, nodes);
        EdgeBorrowMutApi::new(edge, &mut self.edge_service, tracker_api, spec_api)
    }

    fn channel_api_mut(&mut self) -> ChannelBorrowMutApi<'_, CRF> {
        let EditorRepositoryViewMut { channel, .. } = self.repo.view_mut();
        ChannelBorrowMutApi::new(channel.view_mut(), &mut self.channel_service)
    }

    fn node_ui_api_mut(&mut self) -> NodeUiBorrowMutApi<'_, URF::UiNodeRepo> {
        let EditorRepositoryViewMut { ui, .. } = self.repo.view_mut();
        NodeUiBorrowMutApi::new(ui.view_mut().node)
    }

    fn channel_ui_api_mut(&mut self) -> ChannelUiBorrowMutApi<'_, URF::UiChannelRepo> {
        let EditorRepositoryViewMut { ui, .. } = self.repo.view_mut();
        ChannelUiBorrowMutApi::new(ui.view_mut().channel)
    }

    pub fn import_api(&mut self) -> ImportApi<'_, NRF, ER, CRF, URF> {
        ImportApi::new(
            &mut self.node_service,
            &mut self.edge_service,
            &mut self.channel_service,
            &mut self.repo,
            &self.spec_map,
        )
    }

    pub fn export_api(
        &self,
    ) -> ExportApi<
        '_,
        NRF,
        impl SpecByNodeIdQueryApi,
        impl EdgeQueryApi,
        impl ChannelQueryApi,
        impl NodeUiQueryApi,
        impl ChannelUiQueryApi,
    > {
        ExportApi::new(
            ChannelExtApi::borrow(self),
            self.node_api(),
            NodeExtApi::spec_by_node_id(self),
            EdgeExtApi::borrow(self),
            NodeUiExtApi::borrow(self),
            ChannelUiExtApi::borrow(self),
        )
    }
}

pub trait NodeExtApi {
    fn create(&mut self, spec: &NodeSpec) -> Result<NodeId>;
    fn remove(&mut self, id: NodeId) -> Result<()>;

    fn spec_by_spec_id(&self) -> impl SpecBySpecIdQueryApi;
    fn spec_by_node_id(&self) -> impl SpecByNodeIdQueryApi;

    fn tracker(&self) -> impl NodeTrackerQueryApi;
    fn port_state(&self) -> impl PortStateQueryApi;
    fn port_connection<'a>(
        &'a mut self,
        ports_spec: &'a PortsSpec,
    ) -> impl PortConnectionMutApi + 'a;
    fn port_connection_by_node<'a>(
        &'a mut self,
        node_id: NodeId,
    ) -> Result<impl PortConnectionMutApi + 'a>;
    fn parameters_mut(&mut self) -> impl ParameterValueMutApi;
}

impl<NRF, ER, CRF, URF> NodeExtApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn create(&mut self, spec: &NodeSpec) -> Result<NodeId> {
        self.node_api_mut().lifecycle().create(spec)
    }

    fn remove(&mut self, id: NodeId) -> Result<()> {
        //@todo needs refactoring to do without cloning
        let spec = self.spec_by_node_id().spec(id)?.clone();
        self.node_api_mut().lifecycle().remove(&spec, id)
    }

    fn spec_by_spec_id(&self) -> impl SpecBySpecIdQueryApi {
        let EditorRepositoryView { node, .. } = self.repo.view();
        SpecBySpecIdQuery::new(node.view().specs)
    }

    fn spec_by_node_id(&self) -> impl SpecByNodeIdQueryApi {
        let EditorRepositoryView { node, .. } = self.repo.view();
        SpecByNodeIdQuery::new(SpecBySpecIdQuery::new(node.view().specs), node.view().nodes)
    }

    fn tracker(&self) -> impl NodeTrackerQueryApi {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView { nodes, .. } = node.view();
        TrackerApi::new(&self.node_service, nodes)
    }

    fn port_state(&self) -> impl PortStateQueryApi {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView { ports, .. } = node.view();
        PortStateApi::new(ports)
    }

    fn port_connection<'a>(&'a mut self, ports_spec: &'a PortsSpec) -> impl PortConnectionMutApi {
        let EditorRepositoryViewMut { node, channel, .. } = self.repo.view_mut();
        PortConnectionApi::new(
            node.view_mut().ports,
            ports_spec,
            ChannelBorrowMutApi::new(channel.view_mut(), &mut self.channel_service),
        )
    }

    fn port_connection_by_node(&mut self, node_id: NodeId) -> Result<impl PortConnectionMutApi> {
        let EditorRepositoryViewMut { node, channel, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeViewMut {
            nodes,
            specs,
            ports,
            ..
        } = node.view_mut();

        let spec_id = nodes
            .spec_id(&node_id)
            .copied()
            .ok_or_else(|| anyhow!("no mapping between node id {node_id} and spec id exists"))?;
        let spec = specs
            .spec(spec_id)
            .ok_or_else(|| anyhow!("failed to obtain spec {spec_id}"))?;
        let ports_spec = spec
            .ports()
            .as_ref()
            .ok_or_else(|| anyhow!("node {node_id} has no ports spec"))?;

        Ok(PortConnectionApi::new(
            ports,
            ports_spec,
            ChannelBorrowMutApi::new(channel.view_mut(), &mut self.channel_service),
        ))
    }

    fn parameters_mut(&mut self) -> impl ParameterValueMutApi {
        let EditorRepositoryViewMut { node, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeViewMut { parameters, .. } = node.view_mut();
        ParameterValueBorrowMutApi::new(parameters)
    }
}

pub trait NodePortOpsApi: NodeExtApi {
    fn is_external(&self, node_id: NodeId, port_id: NodePortId) -> Result<bool> {
        let port_state = NodeExtApi::port_state(self);
        Ok(port_state
            .node_conns(node_id)
            .find(|(id, _)| **id == port_id)
            .map(|(_, state)| state.is_external())
            .unwrap_or(false))
    }

    fn connect_port(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
        channel_id: ChannelId,
    ) -> Result<()> {
        if self.is_external(node_id, port_id)? {
            return Err(anyhow!(
                "attempted to connect port that is marked as external"
            ));
        }
        let mut port_connection = NodeExtApi::port_connection_by_node(self, node_id)?;
        port_connection.connect(PortConnectionInput::new(node_id, port_id, channel_id))
    }

    fn disconnect_port_connection(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
        channel_id: ChannelId,
    ) -> Result<()> {
        let mut port_connection = NodeExtApi::port_connection_by_node(self, node_id)?;
        port_connection.disconnect(PortConnectionInput::new(node_id, port_id, channel_id))
    }

    fn set_port_external(&mut self, node_id: NodeId, port_id: NodePortId) -> Result<()> {
        let mut port_connection = NodeExtApi::port_connection_by_node(self, node_id)?;
        port_connection.set_external(node_id, port_id)
    }

    fn set_port_internal(&mut self, node_id: NodeId, port_id: NodePortId) -> Result<()> {
        let mut port_connection = NodeExtApi::port_connection_by_node(self, node_id)?;
        port_connection.disconnect_port(node_id, port_id)
    }

    fn internal_connections(&self) -> impl Iterator<Item = (&NodeId, &NodePortId, &ChannelId)>;

    fn connection_views(&self) -> impl Iterator<Item = Result<PortConnectionView<'_>>>;

    fn connection_views_by_kind(
        &self,
        kind: NodePortKind,
    ) -> impl Iterator<Item = Result<PortConnectionView<'_>>> {
        self.connection_views().filter_map(move |res| match res {
            Ok(view) if view.kind == kind => Some(Ok(view)),
            Ok(_) => None,
            Err(err) => Some(Err(err)),
        })
    }
}

struct ConnectionViewIter<'a, NR, SR, I> {
    node_repo: &'a NR,
    spec_repo: &'a SR,
    iter: I,
}

impl<'a, NR, SR, I> ConnectionViewIter<'a, NR, SR, I>
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

impl<'a, NR, SR, I> Iterator for ConnectionViewIter<'a, NR, SR, I>
where
    NR: NodeRepositoryConcept + 'a,
    SR: SpecRepositoryConcept<Spec = NodeSpec, SpecId = beetry_editor_types::id::NodeSpecId> + 'a,
    I: Iterator<Item = (&'a NodeId, &'a NodePortId, &'a ChannelId)>,
{
    type Item = Result<PortConnectionView<'a>>;

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
        Some(Ok(PortConnectionView {
            node_id: *node_id,
            port_id: *port_id,
            channel_id: *channel_id,
            kind: port_spec.kind,
            msg_desc: port_spec.msg_spec.as_str(),
        }))
    }
}

impl<NRF, ER, CRF, URF> NodePortOpsApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn internal_connections(&self) -> impl Iterator<Item = (&NodeId, &NodePortId, &ChannelId)> {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView { ports, .. } = node.view();
        ports
            .iter()
            .flat_map(|(node_id, port_iter): (&NodeId, _)| {
                port_iter
                    .filter_map(
                        move |(port_id, state): (&NodePortId, &PortConnectionState)| match state {
                            PortConnectionState::Internal(conns) => Some((port_id, conns)),
                            PortConnectionState::External => None,
                        },
                    )
                    .flat_map(move |(port_id, conns)| {
                        conns
                            .iter()
                            .map(move |channel_id| (node_id, port_id, channel_id))
                    })
            })
    }

    fn connection_views(&self) -> impl Iterator<Item = Result<PortConnectionView<'_>>> {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView { nodes, specs, .. } = node.view();
        ConnectionViewIter::new(nodes, specs, self.internal_connections())
    }
}

pub trait NodeOpsApi: NodeExtApi + NodeUiExtApi {
    fn create_with_ui(&mut self, spec: &NodeSpec, ui_data: NodeUiData) -> Result<NodeId> {
        let id = NodeExtApi::create(self, spec)?;
        if NodeUiExtApi::create(self, id, ui_data).is_err() {
            NodeExtApi::remove(self, id)?;
        }
        Ok(id)
    }

    fn remove_with_ui(&mut self, id: NodeId) -> Result<()> {
        NodeExtApi::remove(self, id)?;
        NodeUiExtApi::remove(self, id);
        Ok(())
    }
}

impl<T> NodeOpsApi for T where T: NodeExtApi + NodeUiExtApi {}

pub trait EdgeExtApi {
    fn create(&mut self, edge: NodeEdge) -> Result<EdgeId>;
    fn remove(&mut self, id: EdgeId) -> Result<()>;

    fn borrow(&self) -> impl EdgeQueryApi;
}

impl<NRF, ER, CRF, URF> EdgeExtApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn create(&mut self, edge: NodeEdge) -> Result<EdgeId> {
        self.edge_api_mut().create(edge)
    }

    fn remove(&mut self, id: EdgeId) -> Result<()> {
        self.edge_api_mut().remove(id)
    }

    fn borrow(&self) -> impl EdgeQueryApi {
        let EditorRepositoryView { edge, .. } = self.repo.view();
        EdgeBorrowApi::new(edge, &self.edge_service)
    }
}

pub trait NodeUiExtApi {
    fn create(&mut self, id: NodeId, data: NodeUiData) -> Result<()>;
    fn remove(&mut self, id: NodeId) -> Option<NodeUiData>;

    fn update_position(&mut self, id: NodeId, position: Point) -> Result<()>;

    fn borrow(&self) -> impl NodeUiQueryApi;
}

impl<NRF, ER, CRF, URF> NodeUiExtApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn create(&mut self, id: NodeId, data: NodeUiData) -> Result<()> {
        self.node_ui_api_mut().create(id, data)
    }

    fn remove(&mut self, id: NodeId) -> Option<NodeUiData> {
        self.node_ui_api_mut().remove(id)
    }

    fn update_position(&mut self, id: NodeId, position: Point) -> Result<()> {
        self.node_ui_api_mut().update_position(id, position)
    }

    fn borrow(&self) -> impl NodeUiQueryApi {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        NodeUiBorrowApi::new(ui.view().node)
    }
}

pub trait ChannelExtApi {
    fn create(&mut self, spec: &ChannelSpec, config: ChannelConfig) -> Result<ChannelId>;
    fn remove(&mut self, id: ChannelId) -> Option<ChannelData>;

    fn borrow(&self) -> impl ChannelQueryApi;
}

impl<NRF, ER, CRF, URF> ChannelExtApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn create(&mut self, spec: &ChannelSpec, config: ChannelConfig) -> Result<ChannelId> {
        self.channel_api_mut().create(spec, config)
    }

    fn remove(&mut self, id: ChannelId) -> Option<ChannelData> {
        self.channel_api_mut().remove(id)
    }

    fn borrow(&self) -> impl ChannelQueryApi {
        let EditorRepositoryView { channel, .. } = self.repo.view();
        ChannelBorrowApi::new(channel.view())
    }
}

pub trait ChannelUiExtApi {
    fn create(&mut self, id: ChannelId, data: ChannelUiData) -> Result<()>;
    fn remove(&mut self, id: ChannelId) -> Option<ChannelUiData>;

    fn update_position(&mut self, id: ChannelId, position: Point) -> Result<()>;

    fn borrow(&self) -> impl ChannelUiQueryApi;
}

impl<NRF, ER, CRF, URF> ChannelUiExtApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn create(&mut self, id: ChannelId, data: ChannelUiData) -> Result<()> {
        self.channel_ui_api_mut().create(id, data)
    }

    fn remove(&mut self, id: ChannelId) -> Option<ChannelUiData> {
        self.channel_ui_api_mut().remove(id)
    }

    fn update_position(&mut self, id: ChannelId, position: Point) -> Result<()> {
        self.channel_ui_api_mut().update_position(id, position)
    }

    fn borrow(&self) -> impl ChannelUiQueryApi {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        ChannelUiBorrowApi::new(ui.view().channel)
    }
}

pub trait ChannelOpsApi: ChannelExtApi + ChannelUiExtApi {
    fn create_with_ui(
        &mut self,
        spec: &ChannelSpec,
        config: ChannelConfig,
        ui_data: ChannelUiData,
    ) -> Result<ChannelId> {
        let id = ChannelExtApi::create(self, spec, config)?;
        if ChannelUiExtApi::create(self, id, ui_data).is_err() {
            ChannelExtApi::remove(self, id);
        }
        Ok(id)
    }

    fn remove_with_ui(&mut self, id: ChannelId) -> Result<()> {
        ChannelExtApi::remove(self, id);
        ChannelUiExtApi::remove(self, id);
        Ok(())
    }
}

impl<T> ChannelOpsApi for T where T: ChannelExtApi + ChannelUiExtApi {}

pub trait EditorServiceApi:
    NodeExtApi + NodeUiExtApi + EdgeExtApi + ChannelExtApi + ChannelUiExtApi
{
}

impl<NRF, ER, CRF, URF> EditorServiceApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
}

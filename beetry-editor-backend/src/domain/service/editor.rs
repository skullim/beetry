use crate::{
    NodeSpecMap,
    channel::ChannelQueryApi,
    domain::{
        channel::ChannelView,
        edge::EdgeView,
        export::ExportView,
        import::ImportViewMut,
        node::NodeView,
        repository::{
            ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, EditorRepository,
            EditorRepositoryView, EditorRepositoryViewMut, NodeRepositoryConcept,
            NodeRepositoryFacadeConcept, NodeRepositoryFacadeView, NodeRepositoryFacadeViewMut,
            PortStateRepositoryConcept, SpecRepositoryConcept, UiRepositoryFacadeConcept,
        },
        service::{
            channel::{ChannelViewMut, ChannelService},
            edge::{EdgeViewMut, EdgeService},
            node::{self, NodeViewMut, NodeService},
        },
    },
    edge::EdgeQueryApi,
    node::{
        NodeTrackerQueryApi, ParameterValueViewMut, ParameterValueMutApi, PortConnectionApi,
        PortConnectionInput, PortConnectionMutApi, PortConnectionView, PortStateApi,
        PortStateQueryApi, SpecByNodeIdQuery, SpecByNodeIdQueryApi, SpecBySpecIdQuery,
        SpecBySpecIdQueryApi, TrackerApi,
    },
    ui::{
        ChannelUiView, ChannelUiViewMut, ChannelUiQueryApi, NodeUiView,
        NodeUiViewMut, NodeUiQueryApi,
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
    persistence::{EditorStateStore, ValidTree},
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

    fn node_view(&self) -> NodeView<'_, NRF> {
        let EditorRepositoryView { node, .. } = self.repo.view();
        NodeView::new(node.view(), &self.node_service)
    }

    fn node_view_mut(&mut self) -> NodeViewMut<'_, NRF, ER, CRF> {
        let EditorRepositoryViewMut {
            node,
            edge,
            channel,
            ..
        } = self.repo.view_mut();

        NodeViewMut::new(
            node.view_mut(),
            &mut self.node_service,
            edge,
            &mut self.edge_service,
            channel,
            &mut self.channel_service,
        )
    }

    fn edge_view_mut(&mut self) -> EdgeViewMut<'_, ER, NRF> {
        let EditorRepositoryViewMut { node, edge, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeView { nodes, specs, .. } = node.view();
        let tracker_api = node::TrackerApi::new(&self.node_service, nodes);
        let spec_api = node::SpecApi::new(specs, nodes);
        EdgeViewMut::new(edge, &mut self.edge_service, tracker_api, spec_api)
    }

    fn channel_view_mut(&mut self) -> ChannelViewMut<'_, CRF> {
        let EditorRepositoryViewMut { channel, .. } = self.repo.view_mut();
        ChannelViewMut::new(channel.view_mut(), &mut self.channel_service)
    }

    fn node_ui_view_mut(&mut self) -> NodeUiViewMut<'_, URF::UiNodeRepo> {
        let EditorRepositoryViewMut { ui, .. } = self.repo.view_mut();
        NodeUiViewMut::new(ui.view_mut().node)
    }

    fn channel_ui_view_mut(&mut self) -> ChannelUiViewMut<'_, URF::UiChannelRepo> {
        let EditorRepositoryViewMut { ui, .. } = self.repo.view_mut();
        ChannelUiViewMut::new(ui.view_mut().channel)
    }

    fn import_view_mut(&mut self) -> ImportViewMut<'_, NRF, ER, CRF, URF> {
        ImportViewMut::new(
            &mut self.node_service,
            &mut self.edge_service,
            &mut self.channel_service,
            &mut self.repo,
            &self.spec_map,
        )
    }

    fn export_view(
        &self,
    ) -> ExportView<
        '_,
        NRF,
        impl SpecByNodeIdQueryApi,
        impl EdgeQueryApi,
        impl ChannelQueryApi,
        impl NodeUiQueryApi,
        impl ChannelUiQueryApi,
    > {
        ExportView::new(
            ChannelApi::borrow(self),
            self.node_view(),
            NodeApi::spec_by_node_id(self),
            EdgeApi::borrow(self),
            NodeUiApi::borrow(self),
            ChannelUiApi::borrow(self),
        )
    }
}

pub trait ImportApi {
    fn import_project(&mut self, store: EditorStateStore) -> Result<()>;
}

impl<NRF, ER, CRF, URF> ImportApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn import_project(&mut self, store: EditorStateStore) -> Result<()> {
        self.import_view_mut().import_project(store)
    }
}

pub trait ExportApi {
    fn export_project(&self) -> Result<EditorStateStore>;
    fn export_valid_tree(&self) -> Result<ValidTree>;
}

impl<NRF, ER, CRF, URF> ExportApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn export_project(&self) -> Result<EditorStateStore> {
        let mut export_api = self.export_view();
        export_api.export_project()
    }

    fn export_valid_tree(&self) -> Result<ValidTree> {
        let mut export_api = self.export_view();
        export_api.export_valid_tree()
    }
}

pub trait NodeApi {
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

impl<NRF, ER, CRF, URF> NodeApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn create(&mut self, spec: &NodeSpec) -> Result<NodeId> {
        self.node_view_mut().lifecycle().create(spec)
    }

    fn remove(&mut self, id: NodeId) -> Result<()> {
        //@todo needs refactoring to do without cloning
        let spec = self.spec_by_node_id().spec(id)?.clone();
        self.node_view_mut().lifecycle().remove(&spec, id)
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
            ChannelViewMut::new(channel.view_mut(), &mut self.channel_service),
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
            ChannelViewMut::new(channel.view_mut(), &mut self.channel_service),
        ))
    }

    fn parameters_mut(&mut self) -> impl ParameterValueMutApi {
        let EditorRepositoryViewMut { node, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeViewMut { parameters, .. } = node.view_mut();
        ParameterValueViewMut::new(parameters)
    }
}

pub trait NodePortApi: NodeApi {
    fn is_external(&self, node_id: NodeId, port_id: NodePortId) -> Result<bool> {
        let port_state = NodeApi::port_state(self);
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
        let mut port_connection = NodeApi::port_connection_by_node(self, node_id)?;
        port_connection.connect(PortConnectionInput::new(node_id, port_id, channel_id))
    }

    fn disconnect_port_connection(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
        channel_id: ChannelId,
    ) -> Result<()> {
        let mut port_connection = NodeApi::port_connection_by_node(self, node_id)?;
        port_connection.disconnect(PortConnectionInput::new(node_id, port_id, channel_id))
    }

    fn set_port_external(&mut self, node_id: NodeId, port_id: NodePortId) -> Result<()> {
        let mut port_connection = NodeApi::port_connection_by_node(self, node_id)?;
        port_connection.set_external(node_id, port_id)
    }

    fn set_port_internal(&mut self, node_id: NodeId, port_id: NodePortId) -> Result<()> {
        let mut port_connection = NodeApi::port_connection_by_node(self, node_id)?;
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

impl<NRF, ER, CRF, URF> NodePortApi for EditorService<NRF, ER, CRF, URF>
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

pub trait NodeLifecycleApi: NodeApi + NodeUiApi {
    fn create_with_ui(&mut self, spec: &NodeSpec, ui_data: NodeUiData) -> Result<NodeId> {
        let id = NodeApi::create(self, spec)?;
        if NodeUiApi::create(self, id, ui_data).is_err() {
            NodeApi::remove(self, id)?;
        }
        Ok(id)
    }

    fn remove_with_ui(&mut self, id: NodeId) -> Result<()> {
        NodeApi::remove(self, id)?;
        NodeUiApi::remove(self, id);
        Ok(())
    }
}

impl<T> NodeLifecycleApi for T where T: NodeApi + NodeUiApi {}

pub trait EdgeApi {
    fn create(&mut self, edge: NodeEdge) -> Result<EdgeId>;
    fn remove(&mut self, id: EdgeId) -> Result<()>;

    fn borrow(&self) -> impl EdgeQueryApi;
}

impl<NRF, ER, CRF, URF> EdgeApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn create(&mut self, edge: NodeEdge) -> Result<EdgeId> {
        self.edge_view_mut().create(edge)
    }

    fn remove(&mut self, id: EdgeId) -> Result<()> {
        self.edge_view_mut().remove(id)
    }

    fn borrow(&self) -> impl EdgeQueryApi {
        let EditorRepositoryView { edge, .. } = self.repo.view();
        EdgeView::new(edge, &self.edge_service)
    }
}

pub trait NodeUiApi {
    fn create(&mut self, id: NodeId, data: NodeUiData) -> Result<()>;
    fn remove(&mut self, id: NodeId) -> Option<NodeUiData>;

    fn update_position(&mut self, id: NodeId, position: Point) -> Result<()>;

    fn borrow(&self) -> impl NodeUiQueryApi;
}

impl<NRF, ER, CRF, URF> NodeUiApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn create(&mut self, id: NodeId, data: NodeUiData) -> Result<()> {
        self.node_ui_view_mut().create(id, data)
    }

    fn remove(&mut self, id: NodeId) -> Option<NodeUiData> {
        self.node_ui_view_mut().remove(id)
    }

    fn update_position(&mut self, id: NodeId, position: Point) -> Result<()> {
        self.node_ui_view_mut().update_position(id, position)
    }

    fn borrow(&self) -> impl NodeUiQueryApi {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        NodeUiView::new(ui.view().node)
    }
}

pub trait ChannelApi {
    fn create(&mut self, spec: &ChannelSpec, config: ChannelConfig) -> Result<ChannelId>;
    fn remove(&mut self, id: ChannelId) -> Option<ChannelData>;

    fn borrow(&self) -> impl ChannelQueryApi;
}

impl<NRF, ER, CRF, URF> ChannelApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn create(&mut self, spec: &ChannelSpec, config: ChannelConfig) -> Result<ChannelId> {
        self.channel_view_mut().create(spec, config)
    }

    fn remove(&mut self, id: ChannelId) -> Option<ChannelData> {
        self.channel_view_mut().remove(id)
    }

    fn borrow(&self) -> impl ChannelQueryApi {
        let EditorRepositoryView { channel, .. } = self.repo.view();
        ChannelView::new(channel.view())
    }
}

pub trait ChannelUiApi {
    fn create(&mut self, id: ChannelId, data: ChannelUiData) -> Result<()>;
    fn remove(&mut self, id: ChannelId) -> Option<ChannelUiData>;

    fn update_position(&mut self, id: ChannelId, position: Point) -> Result<()>;

    fn borrow(&self) -> impl ChannelUiQueryApi;
}

impl<NRF, ER, CRF, URF> ChannelUiApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn create(&mut self, id: ChannelId, data: ChannelUiData) -> Result<()> {
        self.channel_ui_view_mut().create(id, data)
    }

    fn remove(&mut self, id: ChannelId) -> Option<ChannelUiData> {
        self.channel_ui_view_mut().remove(id)
    }

    fn update_position(&mut self, id: ChannelId, position: Point) -> Result<()> {
        self.channel_ui_view_mut().update_position(id, position)
    }

    fn borrow(&self) -> impl ChannelUiQueryApi {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        ChannelUiView::new(ui.view().channel)
    }
}

pub trait ChannelLifecycleApi: ChannelApi + ChannelUiApi {
    fn create_with_ui(
        &mut self,
        spec: &ChannelSpec,
        config: ChannelConfig,
        ui_data: ChannelUiData,
    ) -> Result<ChannelId> {
        let id = ChannelApi::create(self, spec, config)?;
        if ChannelUiApi::create(self, id, ui_data).is_err() {
            ChannelApi::remove(self, id);
        }
        Ok(id)
    }

    fn remove_with_ui(&mut self, id: ChannelId) -> Result<()> {
        ChannelApi::remove(self, id);
        ChannelUiApi::remove(self, id);
        Ok(())
    }
}

impl<T> ChannelLifecycleApi for T where T: ChannelApi + ChannelUiApi {}

pub trait EditorServiceApi:
    NodeApi + NodeUiApi + EdgeApi + ChannelApi + ChannelUiApi
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

use crate::{
    NodeSpecMap,
    channel::{ChannelQueryView, ChannelView},
    edge::{EdgeQueryView, EdgeView},
    node::{
        NodeTrackerQuery, NodeView, ParameterValueMut, ParameterValueQuery,
        ParameterValueQueryView, ParameterValueViewMut, PortConnectionQuery,
        PortConnectionQueryView, PortSpecQuery, PortStateQuery, PortStateQueryView,
        SpecByNodeIdQuery, SpecByNodeIdQueryView, SpecBySpecIdQuery, SpecBySpecIdQueryView,
        TrackerView,
    },
    repository::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, EditorRepository,
        EditorRepositoryView, EditorRepositoryViewMut, NodeRepositoryFacadeConcept,
        NodeRepositoryFacadeView, NodeRepositoryFacadeViewMut, UiRepositoryFacadeConcept,
    },
    service::{
        channel::{ChannelService, ChannelViewMut},
        edge::{EdgeService, EdgeViewMut},
        export::ExportView,
        import::ImportViewMut,
        node::{self, NodeService, NodeViewMut},
    },
    ui::{
        ChannelUiQuery, ChannelUiQueryView, ChannelUiViewMut, NodeUiQuery, NodeUiQueryView,
        NodeUiViewMut, PortConnectionUiQuery, PortConnectionUiQueryView,
        PortConnectionUiStateViewMut,
    },
};
use anyhow::{Result, bail};
use beetry_editor_types::{
    id::{ChannelId, EdgeId, NodeId, NodePortId, PortConnectionId},
    output::{
        channel::{ChannelConfigInput, ChannelConfigUpdate, ChannelData},
        edge::NodeEdge,
        node::PortState,
        ui::{ChannelUiData, NodeUiData, Point, PortConnectionUiData},
    },
    persistence::{EditorStateStore, ValidTree},
    spec::{
        channel::ChannelSpec,
        node::{NodePortKind, NodeSpec},
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
        let tracker_view = node::TrackerView::new(&self.node_service, nodes);
        let spec_view = node::SpecView::new(specs, nodes);
        EdgeViewMut::new(edge, &mut self.edge_service, tracker_view, spec_view)
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

    fn port_connection_ui_view_mut(
        &mut self,
    ) -> PortConnectionUiStateViewMut<'_, URF::UiPortConnectionRepo, NRF::PortConnectionRepo> {
        let EditorRepositoryViewMut { node, ui, .. } = self.repo.view_mut();
        PortConnectionUiStateViewMut::new(
            ui.view_mut().port_connection,
            node.view().port_connections,
        )
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
        impl SpecByNodeIdQuery,
        impl EdgeQueryView,
        impl ChannelQueryView,
        impl NodeUiQuery,
        impl ChannelUiQuery,
        URF::UiPortConnectionRepo,
    > {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        ExportView::new(
            ChannelApi::query(self),
            self.node_view(),
            NodeApi::spec_by_node_id(self),
            EdgeApi::query(self),
            NodeUiApi::query(self),
            ChannelUiApi::query(self),
            ui.view().port_connection,
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

    fn spec_by_spec_id(&self) -> impl SpecBySpecIdQuery;
    fn spec_by_node_id(&self) -> impl SpecByNodeIdQuery;

    fn tracker(&self) -> impl NodeTrackerQuery;

    fn port_state_query(&self) -> impl PortStateQuery;

    fn parameters(&self) -> impl ParameterValueQuery;
    fn parameters_mut(&mut self) -> impl ParameterValueMut;
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
        let spec = self.spec_by_node_id().spec(id)?.clone();
        let EditorRepositoryViewMut {
            node,
            edge,
            channel,
            ..
        } = self.repo.view_mut();
        let mut node_view = NodeViewMut::new(
            node.view_mut(),
            &mut self.node_service,
            edge,
            &mut self.edge_service,
            channel,
            &mut self.channel_service,
        );
        node_view.lifecycle().remove(&spec, id)
    }

    fn spec_by_spec_id(&self) -> impl SpecBySpecIdQuery {
        let EditorRepositoryView { node, .. } = self.repo.view();
        SpecBySpecIdQueryView::new(node.view().specs)
    }

    fn spec_by_node_id(&self) -> impl SpecByNodeIdQuery {
        let EditorRepositoryView { node, .. } = self.repo.view();
        SpecByNodeIdQueryView::new(
            SpecBySpecIdQueryView::new(node.view().specs),
            node.view().nodes,
        )
    }

    fn tracker(&self) -> impl NodeTrackerQuery {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView { nodes, .. } = node.view();
        TrackerView::new(&self.node_service, nodes)
    }

    fn port_state_query(&self) -> impl PortStateQuery {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView { ports, .. } = node.view();
        PortStateQueryView { repo: ports }
    }

    fn parameters(&self) -> impl ParameterValueQuery {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView { parameters, .. } = node.view();
        ParameterValueQueryView::new(parameters)
    }

    fn parameters_mut(&mut self) -> impl ParameterValueMut {
        let EditorRepositoryViewMut { node, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeViewMut { parameters, .. } = node.view_mut();
        ParameterValueViewMut::new(parameters)
    }
}

pub trait PortApi: NodeApi {
    fn port_order(&self, kind: NodePortKind, node_id: NodeId, port_id: NodePortId)
    -> Result<usize>;

    fn connect_port(&mut self, conn_id: PortConnectionId) -> Result<()>;

    fn disconnect(&mut self, conn_id: PortConnectionId) -> Result<()>;

    fn set_state(&mut self, node_id: NodeId, port_id: NodePortId, state: PortState) -> Result<()>;

    fn connections_query(&self) -> impl PortConnectionQuery;
}

impl<NRF, ER, CRF, URF> PortApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn port_order(
        &self,
        kind: NodePortKind,
        node_id: NodeId,
        port_id: NodePortId,
    ) -> Result<usize> {
        self.node_view()
            .port_spec_query()
            .port_order(kind, node_id, port_id)
    }

    fn connect_port(&mut self, id: PortConnectionId) -> Result<()> {
        let EditorRepositoryViewMut {
            node,
            edge,
            channel,
            ..
        } = self.repo.view_mut();

        let mut node_view = NodeViewMut::new(
            node.view_mut(),
            &mut self.node_service,
            edge,
            &mut self.edge_service,
            channel,
            &mut self.channel_service,
        );
        node_view.port_connection().connect_port(id)
    }

    fn disconnect(&mut self, conn_id: PortConnectionId) -> Result<()> {
        let EditorRepositoryViewMut {
            node,
            edge,
            channel,
            ..
        } = self.repo.view_mut();

        let mut node_view = NodeViewMut::new(
            node.view_mut(),
            &mut self.node_service,
            edge,
            &mut self.edge_service,
            channel,
            &mut self.channel_service,
        );
        node_view.port_connection().disconnect(conn_id)
    }

    fn set_state(&mut self, node_id: NodeId, port_id: NodePortId, state: PortState) -> Result<()> {
        let EditorRepositoryViewMut {
            node,
            edge,
            channel,
            ..
        } = self.repo.view_mut();

        let mut node_view = NodeViewMut::new(
            node.view_mut(),
            &mut self.node_service,
            edge,
            &mut self.edge_service,
            channel,
            &mut self.channel_service,
        );
        node_view.port_state().set_state(node_id, port_id, state)
    }

    fn connections_query(&self) -> impl PortConnectionQuery {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView {
            port_connections, ..
        } = node.view();
        PortConnectionQueryView::new(port_connections)
    }
}

pub trait NodeLifecycleApi: NodeApi + NodeUiApi {
    fn create(&mut self, spec: &NodeSpec, ui_data: NodeUiData) -> Result<NodeId> {
        let id = NodeApi::create(self, spec)?;
        if NodeUiApi::create(self, id, ui_data).is_err() {
            NodeApi::remove(self, id)?;
        }
        Ok(id)
    }

    fn remove(&mut self, id: NodeId) -> Result<()> {
        NodeApi::remove(self, id)?;
        NodeUiApi::remove(self, id);
        Ok(())
    }
}

impl<T> NodeLifecycleApi for T where T: NodeApi + NodeUiApi {}

pub trait EdgeApi {
    fn create(&mut self, edge: NodeEdge) -> Result<EdgeId>;
    fn remove(&mut self, id: EdgeId) -> Result<()>;

    fn query(&self) -> impl EdgeQueryView;
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

    fn query(&self) -> impl EdgeQueryView {
        let EditorRepositoryView { edge, .. } = self.repo.view();
        EdgeView::new(edge, &self.edge_service)
    }
}

pub trait NodeUiApi {
    fn create(&mut self, id: NodeId, data: NodeUiData) -> Result<()>;
    fn remove(&mut self, id: NodeId) -> Option<NodeUiData>;

    fn update_position(&mut self, id: NodeId, position: Point) -> Result<()>;

    fn query(&self) -> impl NodeUiQuery;
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

    fn query(&self) -> impl NodeUiQuery {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        NodeUiQueryView::new(ui.view().node)
    }
}

pub trait ChannelApi {
    fn create(&mut self, spec: &ChannelSpec, input: ChannelConfigInput) -> Result<ChannelId>;
    fn remove(&mut self, id: ChannelId) -> Option<ChannelData>;
    fn update_config(&mut self, id: ChannelId, update: ChannelConfigUpdate) -> Result<()>;

    fn query(&self) -> impl ChannelQueryView;
}

impl<NRF, ER, CRF, URF> ChannelApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn create(&mut self, spec: &ChannelSpec, input: ChannelConfigInput) -> Result<ChannelId> {
        self.channel_view_mut().create(spec, input)
    }

    fn remove(&mut self, id: ChannelId) -> Option<ChannelData> {
        self.channel_view_mut().remove(id)
    }

    fn update_config(&mut self, id: ChannelId, update: ChannelConfigUpdate) -> Result<()> {
        self.channel_view_mut().update_config(id, update)
    }

    fn query(&self) -> impl ChannelQueryView {
        let EditorRepositoryView { channel, .. } = self.repo.view();
        ChannelView::new(channel.view())
    }
}

pub trait ChannelUiApi {
    fn create(&mut self, id: ChannelId, data: ChannelUiData) -> Result<()>;
    fn remove(&mut self, id: ChannelId) -> Option<ChannelUiData>;

    fn update_position(&mut self, id: ChannelId, position: Point) -> Result<()>;

    fn query(&self) -> impl ChannelUiQuery;
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

    fn query(&self) -> impl ChannelUiQuery {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        ChannelUiQueryView::new(ui.view().channel)
    }
}

pub trait ChannelLifecycleApi: ChannelApi + ChannelUiApi + PortLifecycleApi {
    fn create(
        &mut self,
        spec: &ChannelSpec,
        input: ChannelConfigInput,
        ui_data: ChannelUiData,
    ) -> Result<ChannelId> {
        let id = ChannelApi::create(self, spec, input)?;
        if ChannelUiApi::create(self, id, ui_data).is_err() {
            ChannelApi::remove(self, id);
        }
        Ok(id)
    }

    fn remove(&mut self, id: ChannelId) -> Result<()> {
        let to_disconnect: Vec<_> = self
            .connections_query()
            .all_connections()
            .filter(|conn| conn.channel_id == id)
            .collect();

        for conn_id in to_disconnect {
            PortLifecycleApi::disconnect(self, conn_id)?;
        }

        let removed_channel = ChannelApi::remove(self, id);
        let removed_channel_ui = ChannelUiApi::remove(self, id);
        if removed_channel.is_none() || removed_channel_ui.is_none() {
            bail!("channel {id} does not exist");
        }

        Ok(())
    }
}

impl<T> ChannelLifecycleApi for T where T: ChannelApi + ChannelUiApi + PortLifecycleApi {}

pub trait PortConnectionUiApi {
    fn create(&mut self, id: PortConnectionId, ui_data: PortConnectionUiData) -> Result<()>;
    fn remove(&mut self, id: PortConnectionId) -> Result<()>;

    fn update_data(&mut self, id: PortConnectionId, ui_data: PortConnectionUiData) -> Result<()>;

    fn query(&self) -> impl PortConnectionUiQuery;
}

impl<NRF, ER, CRF, URF> PortConnectionUiApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn create(&mut self, id: PortConnectionId, ui_data: PortConnectionUiData) -> Result<()> {
        self.port_connection_ui_view_mut().create(id, ui_data)
    }

    fn remove(&mut self, id: PortConnectionId) -> Result<()> {
        self.port_connection_ui_view_mut().remove(id);
        Ok(())
    }

    fn update_data(&mut self, id: PortConnectionId, ui_data: PortConnectionUiData) -> Result<()> {
        self.port_connection_ui_view_mut().update(id, ui_data)
    }

    fn query(&self) -> impl PortConnectionUiQuery {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        PortConnectionUiQueryView::new(ui.view().port_connection)
    }
}

pub trait PortLifecycleApi: PortApi + PortConnectionUiApi {
    fn connect_port(&mut self, id: PortConnectionId, ui_data: PortConnectionUiData) -> Result<()> {
        PortApi::connect_port(self, id)?;
        if PortConnectionUiApi::create(self, id, ui_data).is_err() {
            PortApi::disconnect(self, id)?;
        }
        Ok(())
    }

    fn disconnect(&mut self, id: PortConnectionId) -> Result<()> {
        PortApi::disconnect(self, id)?;
        PortConnectionUiApi::remove(self, id)
    }
}

impl<T> PortLifecycleApi for T where T: PortApi + PortConnectionUiApi {}

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
        EditorRepository, EditorRepositoryView, EditorRepositoryViewMut, NodeRepositoryFacadeView,
        NodeRepositoryFacadeViewMut,
    },
    service::{
        channel::{ChannelService, ChannelViewMut},
        edge::{EdgeService, EdgeViewMut, OnNodeRemovalServiceApi},
        export::ExportView,
        import::ImportViewMut,
        node::{self, NodeService, PortConnectionViewMut, PortStateViewMut},
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

pub struct EditorService {
    node_service: NodeService,
    edge_service: EdgeService,
    channel_service: ChannelService,
    repo: EditorRepository,
    spec_map: NodeSpecMap,
}

impl EditorService {
    #[must_use]
    pub fn new(spec_map: NodeSpecMap) -> Self {
        Self {
            node_service: NodeService::new(),
            edge_service: EdgeService::new(),
            channel_service: ChannelService::new(),
            repo: EditorRepository::new(),
            spec_map,
        }
    }

    fn node_view(&self) -> NodeView<'_> {
        let EditorRepositoryView { node, .. } = self.repo.view();
        NodeView::new(node, &self.node_service)
    }

    fn edge_view_mut(&mut self) -> EdgeViewMut<'_> {
        let EditorRepositoryViewMut { node, edge, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeViewMut {
            node: nodes,
            spec: specs,
            ..
        } = node;
        let tracker_view = node::TrackerView::new(&self.node_service, nodes);
        let spec_view = node::SpecView::new(specs, nodes);
        EdgeViewMut::new(edge, &mut self.edge_service, tracker_view, spec_view)
    }

    fn node_lifecycle_view_mut(&mut self) -> node::NodeLifecycleView<'_> {
        let EditorRepositoryViewMut {
            node,
            edge,
            channel,
            ..
        } = self.repo.view_mut();
        node::NodeLifecycleView {
            node_service: &mut self.node_service,
            channel_service: &mut self.channel_service,
            node_facade_view: node,
            channel_repo: channel.channel,
            channel_spec_repo: channel.spec,
            edge_removal_service_api: OnNodeRemovalServiceApi::new(&mut self.edge_service, edge),
        }
    }

    fn port_connection_view_mut(&mut self) -> PortConnectionViewMut<'_> {
        let EditorRepositoryViewMut { node, channel, .. } = self.repo.view_mut();
        PortConnectionViewMut {
            node_repo: node.node,
            spec_repo: node.spec,
            port_conn_repo: node.port_connection,
            port_state_repo: node.port_state,
            channel_repo: channel.channel,
            channel_spec_repo: channel.spec,
            channel_service: &mut self.channel_service,
        }
    }

    fn port_state_view_mut(&mut self) -> PortStateViewMut<'_> {
        let EditorRepositoryViewMut { node, channel, .. } = self.repo.view_mut();
        PortStateViewMut {
            node_repo: node.node,
            spec_repo: node.spec,
            port_conn_repo: node.port_connection,
            port_state_repo: node.port_state,
            channel_repo: channel.channel,
            channel_spec_repo: channel.spec,
            channel_service: &mut self.channel_service,
        }
    }

    fn channel_view_mut(&mut self) -> ChannelViewMut<'_> {
        let EditorRepositoryViewMut { channel, .. } = self.repo.view_mut();
        ChannelViewMut::new(channel, &mut self.channel_service)
    }

    fn node_ui_view_mut(&mut self) -> NodeUiViewMut<'_> {
        let EditorRepositoryViewMut { ui, .. } = self.repo.view_mut();
        NodeUiViewMut::new(ui.node)
    }

    fn channel_ui_view_mut(&mut self) -> ChannelUiViewMut<'_> {
        let EditorRepositoryViewMut { ui, .. } = self.repo.view_mut();
        ChannelUiViewMut::new(ui.channel)
    }

    fn port_connection_ui_view_mut(&mut self) -> PortConnectionUiStateViewMut<'_> {
        let EditorRepositoryViewMut { node, ui, .. } = self.repo.view_mut();
        PortConnectionUiStateViewMut::new(ui.port_connection, node.port_connection)
    }

    fn import_view_mut(&mut self) -> ImportViewMut<'_> {
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
        impl SpecByNodeIdQuery,
        impl EdgeQueryView,
        impl ChannelQueryView,
        impl NodeUiQuery,
        impl ChannelUiQuery,
    > {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        ExportView::new(
            ChannelQueryApi::query(self),
            self.node_view(),
            NodeQueryApi::spec_by_node_id(self),
            EdgeQueryApi::query(self),
            NodeUiQueryApi::query(self),
            ChannelUiQueryApi::query(self),
            ui.port_connection,
        )
    }
}

pub trait ImportApi {
    fn import_project(&mut self, store: EditorStateStore) -> Result<()>;
}

impl ImportApi for EditorService {
    fn import_project(&mut self, store: EditorStateStore) -> Result<()> {
        self.import_view_mut().import_project(store)
    }
}

pub trait ExportApi {
    fn export_project(&self) -> Result<EditorStateStore>;
    fn export_valid_tree(&self) -> Result<ValidTree>;
}

impl ExportApi for EditorService {
    fn export_project(&self) -> Result<EditorStateStore> {
        let export_api = self.export_view();
        export_api.export_project()
    }

    fn export_valid_tree(&self) -> Result<ValidTree> {
        let export_api = self.export_view();
        export_api.export_valid_tree()
    }
}

pub trait NodeQueryApi {
    fn spec_by_spec_id(&self) -> impl SpecBySpecIdQuery;
    fn spec_by_node_id(&self) -> impl SpecByNodeIdQuery;
    fn tracker(&self) -> impl NodeTrackerQuery;
}

impl NodeQueryApi for EditorService {
    fn spec_by_spec_id(&self) -> impl SpecBySpecIdQuery {
        let EditorRepositoryView { node, .. } = self.repo.view();
        SpecBySpecIdQueryView::new(node.spec)
    }

    fn spec_by_node_id(&self) -> impl SpecByNodeIdQuery {
        let EditorRepositoryView { node, .. } = self.repo.view();
        SpecByNodeIdQueryView::new(SpecBySpecIdQueryView::new(node.spec), node.node)
    }

    fn tracker(&self) -> impl NodeTrackerQuery {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView { node: nodes, .. } = node;
        TrackerView::new(&self.node_service, nodes)
    }
}

pub trait NodeCommandApi {
    fn create(&mut self, spec: &NodeSpec) -> Result<NodeId>;
    fn remove(&mut self, id: NodeId) -> Result<()>;
}

impl NodeCommandApi for EditorService {
    fn create(&mut self, spec: &NodeSpec) -> Result<NodeId> {
        let mut lifecycle = self.node_lifecycle_view_mut();
        lifecycle.create(spec)
    }

    fn remove(&mut self, id: NodeId) -> Result<()> {
        let spec = self.spec_by_node_id().spec(id)?.clone();
        let mut lifecycle = self.node_lifecycle_view_mut();
        lifecycle.remove(&spec, id)
    }
}

pub trait ParameterQueryApi {
    fn parameters(&self) -> impl ParameterValueQuery;
}

impl ParameterQueryApi for EditorService {
    fn parameters(&self) -> impl ParameterValueQuery {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView {
            parameter: parameters,
            ..
        } = node;
        ParameterValueQueryView::new(parameters)
    }
}

pub trait ParameterCommandApi {
    fn parameters_mut(&mut self) -> impl ParameterValueMut;
}

impl ParameterCommandApi for EditorService {
    fn parameters_mut(&mut self) -> impl ParameterValueMut {
        let EditorRepositoryViewMut { node, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeViewMut {
            parameter: parameters,
            ..
        } = node;
        ParameterValueViewMut::new(parameters)
    }
}

pub trait PortQueryApi {
    fn port_state_query(&self) -> impl PortStateQuery;
    fn port_order(&self, kind: NodePortKind, node_id: NodeId, port_id: NodePortId)
    -> Result<usize>;
    fn connections_query(&self) -> impl PortConnectionQuery;
}

impl PortQueryApi for EditorService {
    fn port_state_query(&self) -> impl PortStateQuery {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView {
            port_state: ports, ..
        } = node;
        PortStateQueryView { repo: ports }
    }

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

    fn connections_query(&self) -> impl PortConnectionQuery {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView {
            port_connection: port_connections,
            ..
        } = node;
        PortConnectionQueryView::new(port_connections)
    }
}

pub trait PortCommandApi {
    fn connect_port(&mut self, conn_id: PortConnectionId) -> Result<()>;
    fn disconnect(&mut self, conn_id: PortConnectionId) -> Result<()>;
    fn set_state(&mut self, node_id: NodeId, port_id: NodePortId, state: PortState) -> Result<()>;
}

impl PortCommandApi for EditorService {
    fn connect_port(&mut self, id: PortConnectionId) -> Result<()> {
        let mut port_connection = self.port_connection_view_mut();
        port_connection.connect_port(id)
    }

    fn disconnect(&mut self, conn_id: PortConnectionId) -> Result<()> {
        let mut port_connection = self.port_connection_view_mut();
        port_connection.disconnect(conn_id)
    }

    fn set_state(&mut self, node_id: NodeId, port_id: NodePortId, state: PortState) -> Result<()> {
        let mut port_state = self.port_state_view_mut();
        port_state.set_state(node_id, port_id, state)
    }
}

pub trait NodeLifecycleApi: NodeCommandApi + NodeUiCommandApi {
    fn create(&mut self, spec: &NodeSpec, ui_data: NodeUiData) -> Result<NodeId> {
        let id = NodeCommandApi::create(self, spec)?;
        if let Err(e) = NodeUiCommandApi::create(self, id, ui_data) {
            NodeCommandApi::remove(self, id)?;
            return Err(e);
        }
        Ok(id)
    }

    fn remove(&mut self, id: NodeId) -> Result<()> {
        NodeCommandApi::remove(self, id)?;
        NodeUiCommandApi::remove(self, id);
        Ok(())
    }
}

impl<T> NodeLifecycleApi for T where T: NodeCommandApi + NodeUiCommandApi {}

pub trait EdgeQueryApi {
    fn query(&self) -> impl EdgeQueryView;
}

impl EdgeQueryApi for EditorService {
    fn query(&self) -> impl EdgeQueryView {
        let EditorRepositoryView { edge, .. } = self.repo.view();
        EdgeView::new(edge, &self.edge_service)
    }
}

pub trait EdgeCommandApi {
    fn create(&mut self, edge: NodeEdge) -> Result<EdgeId>;
    fn remove(&mut self, id: EdgeId) -> Result<()>;
}

impl EdgeCommandApi for EditorService {
    fn create(&mut self, edge: NodeEdge) -> Result<EdgeId> {
        self.edge_view_mut().create(edge)
    }

    fn remove(&mut self, id: EdgeId) -> Result<()> {
        self.edge_view_mut().remove(id)
    }
}

pub trait NodeUiQueryApi {
    fn query(&self) -> impl NodeUiQuery;
}

impl NodeUiQueryApi for EditorService {
    fn query(&self) -> impl NodeUiQuery {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        NodeUiQueryView::new(ui.node)
    }
}

pub trait NodeUiCommandApi {
    fn create(&mut self, id: NodeId, data: NodeUiData) -> Result<()>;
    fn remove(&mut self, id: NodeId) -> Option<NodeUiData>;
    fn update_position(&mut self, id: NodeId, position: Point) -> Result<()>;
}

impl NodeUiCommandApi for EditorService {
    fn create(&mut self, id: NodeId, data: NodeUiData) -> Result<()> {
        self.node_ui_view_mut().create(id, data);
        Ok(())
    }

    fn remove(&mut self, id: NodeId) -> Option<NodeUiData> {
        self.node_ui_view_mut().remove(id)
    }

    fn update_position(&mut self, id: NodeId, position: Point) -> Result<()> {
        self.node_ui_view_mut().update_position(id, position)
    }
}

pub trait ChannelQueryApi {
    fn query(&self) -> impl ChannelQueryView;
}

impl ChannelQueryApi for EditorService {
    fn query(&self) -> impl ChannelQueryView {
        let EditorRepositoryView { channel, .. } = self.repo.view();
        ChannelView::new(channel)
    }
}

pub trait ChannelCommandApi {
    fn create(&mut self, spec: &ChannelSpec, input: ChannelConfigInput) -> ChannelId;
    fn remove(&mut self, id: ChannelId) -> Option<ChannelData>;
    fn update_config(&mut self, id: ChannelId, update: ChannelConfigUpdate) -> Result<()>;
}

impl ChannelCommandApi for EditorService {
    fn create(&mut self, spec: &ChannelSpec, input: ChannelConfigInput) -> ChannelId {
        self.channel_view_mut().create(spec, input)
    }

    fn remove(&mut self, id: ChannelId) -> Option<ChannelData> {
        self.channel_view_mut().remove(id)
    }

    fn update_config(&mut self, id: ChannelId, update: ChannelConfigUpdate) -> Result<()> {
        self.channel_view_mut().update_config(id, update)
    }
}

pub trait ChannelUiQueryApi {
    fn query(&self) -> impl ChannelUiQuery;
}

impl ChannelUiQueryApi for EditorService {
    fn query(&self) -> impl ChannelUiQuery {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        ChannelUiQueryView::new(ui.channel)
    }
}

pub trait ChannelUiCommandApi {
    fn create(&mut self, id: ChannelId, data: ChannelUiData) -> Result<()>;
    fn remove(&mut self, id: ChannelId) -> Option<ChannelUiData>;
    fn update_position(&mut self, id: ChannelId, position: Point) -> Result<()>;
}

impl ChannelUiCommandApi for EditorService {
    fn create(&mut self, id: ChannelId, data: ChannelUiData) -> Result<()> {
        self.channel_ui_view_mut().create(id, data);
        Ok(())
    }

    fn remove(&mut self, id: ChannelId) -> Option<ChannelUiData> {
        self.channel_ui_view_mut().remove(id)
    }

    fn update_position(&mut self, id: ChannelId, position: Point) -> Result<()> {
        self.channel_ui_view_mut().update_position(id, position)
    }
}

pub trait ChannelLifecycleApi:
    ChannelCommandApi + ChannelUiCommandApi + PortLifecycleApi + PortQueryApi
{
    fn create(
        &mut self,
        spec: &ChannelSpec,
        input: ChannelConfigInput,
        ui_data: ChannelUiData,
    ) -> Result<ChannelId> {
        let id = ChannelCommandApi::create(self, spec, input);
        if let Err(e) = ChannelUiCommandApi::create(self, id, ui_data) {
            ChannelCommandApi::remove(self, id);
            return Err(e);
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

        let removed_channel = ChannelCommandApi::remove(self, id);
        let removed_channel_ui = ChannelUiCommandApi::remove(self, id);
        if removed_channel.is_none() || removed_channel_ui.is_none() {
            bail!("channel {id} does not exist");
        }

        Ok(())
    }
}

impl<T> ChannelLifecycleApi for T where
    T: ChannelCommandApi + ChannelUiCommandApi + PortLifecycleApi + PortQueryApi
{
}

pub trait PortConnectionUiQueryApi {
    fn query(&self) -> impl PortConnectionUiQuery;
}

impl PortConnectionUiQueryApi for EditorService {
    fn query(&self) -> impl PortConnectionUiQuery {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        PortConnectionUiQueryView::new(ui.port_connection)
    }
}

pub trait PortConnectionUiCommandApi {
    fn create(&mut self, id: PortConnectionId, ui_data: PortConnectionUiData) -> Result<()>;
    fn remove(&mut self, id: PortConnectionId) -> Result<()>;
    fn update_data(&mut self, id: PortConnectionId, ui_data: PortConnectionUiData) -> Result<()>;
}

impl PortConnectionUiCommandApi for EditorService {
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
}

pub trait PortLifecycleApi: PortCommandApi + PortConnectionUiCommandApi {
    fn connect_port(&mut self, id: PortConnectionId, ui_data: PortConnectionUiData) -> Result<()> {
        PortCommandApi::connect_port(self, id)?;
        if let Err(e) = PortConnectionUiCommandApi::create(self, id, ui_data) {
            PortCommandApi::disconnect(self, id)?;
            return Err(e);
        }
        Ok(())
    }

    fn disconnect(&mut self, id: PortConnectionId) -> Result<()> {
        PortCommandApi::disconnect(self, id)?;
        PortConnectionUiCommandApi::remove(self, id)
    }
}

impl<T> PortLifecycleApi for T where T: PortCommandApi + PortConnectionUiCommandApi {}

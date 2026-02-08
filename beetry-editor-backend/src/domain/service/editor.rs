use crate::{
    NodeSpecMap,
    domain::{
        channel::{ChannelQueryView, ChannelView},
        edge::{EdgeQueryView, EdgeView},
        node::{
            NodeTrackerQueryView, NodeView, ParameterValueMut, ParameterValueViewMut,
            PortConnectionDataView, PortStateQueryApi, SpecByNodeIdQuery,
            SpecByNodeIdQueryView, SpecBySpecIdQuery, SpecBySpecIdQueryView, TrackerView,
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
            ChannelUiQueryApi, ChannelUiView, ChannelUiViewMut, NodeUiQueryApi, NodeUiView,
            NodeUiViewMut,
        },
    },
};
use anyhow::{Result, bail};
use beetry_editor_types::{
    id::{ChannelId, EdgeId, NodeId, NodePortId},
    output::{
        channel::{ChannelConfig, ChannelData},
        edge::NodeEdge,
        ui::{ChannelUiData, NodeUiData, Point},
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
        impl SpecByNodeIdQueryView,
        impl EdgeQueryView,
        impl ChannelQueryView,
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

    fn spec_by_spec_id(&self) -> impl SpecBySpecIdQueryView;
    fn spec_by_node_id(&self) -> impl SpecByNodeIdQueryView;

    fn tracker(&self) -> impl NodeTrackerQueryView;

    fn port_state(&self) -> impl PortStateQueryApi;

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
        //@todo needs refactoring to do without cloning
        let spec = self.spec_by_node_id().spec(id)?.clone();
        self.node_view_mut().lifecycle().remove(&spec, id)
    }

    fn spec_by_spec_id(&self) -> impl SpecBySpecIdQueryView {
        let EditorRepositoryView { node, .. } = self.repo.view();
        SpecBySpecIdQuery::new(node.view().specs)
    }

    fn spec_by_node_id(&self) -> impl SpecByNodeIdQueryView {
        let EditorRepositoryView { node, .. } = self.repo.view();
        SpecByNodeIdQuery::new(SpecBySpecIdQuery::new(node.view().specs), node.view().nodes)
    }

    fn tracker(&self) -> impl NodeTrackerQueryView {
        let EditorRepositoryView { node, .. } = self.repo.view();
        let NodeRepositoryFacadeView { nodes, .. } = node.view();
        TrackerView::new(&self.node_service, nodes)
    }

    fn port_state(&self) -> impl PortStateQueryApi {
        self.node_view().into_port_state()
    }

    fn parameters_mut(&mut self) -> impl ParameterValueMut {
        let EditorRepositoryViewMut { node, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeViewMut { parameters, .. } = node.view_mut();
        ParameterValueViewMut::new(parameters)
    }
}

pub trait NodePortApi: NodeApi {
    fn is_external(&self, node_id: NodeId, port_id: NodePortId) -> Result<bool>;

    fn connect_port(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
        channel_id: ChannelId,
    ) -> Result<()>;

    fn disconnect_port(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
        channel_id: ChannelId,
    ) -> Result<()>;

    fn set_port_external(&mut self, node_id: NodeId, port_id: NodePortId) -> Result<()>;

    fn set_port_internal(&mut self, node_id: NodeId, port_id: NodePortId) -> Result<()>;

    fn internal_connections(&self) -> impl Iterator<Item = (&NodeId, &NodePortId, &ChannelId)>;

    fn connection_views(&self) -> impl Iterator<Item = Result<PortConnectionDataView<'_>>>;

    fn connection_views_by_kind(
        &self,
        kind: NodePortKind,
    ) -> impl Iterator<Item = Result<PortConnectionDataView<'_>>>;
}

impl<NRF, ER, CRF, URF> NodePortApi for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn is_external(&self, node_id: NodeId, port_id: NodePortId) -> Result<bool> {
        self.node_view().port_state().is_external(node_id, port_id)
    }

    fn connect_port(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
        channel_id: ChannelId,
    ) -> Result<()> {
        let mut node = self.node_view_mut();
        node.port_state().connect_port(node_id, port_id, channel_id)
    }

    fn disconnect_port(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
        channel_id: ChannelId,
    ) -> Result<()> {
        let mut node = self.node_view_mut();
        node.port_state()
            .disconnect_port(node_id, port_id, channel_id)
    }

    fn set_port_external(&mut self, node_id: NodeId, port_id: NodePortId) -> Result<()> {
        let mut node = self.node_view_mut();
        node.port_state().set_port_external(node_id, port_id)
    }

    fn set_port_internal(&mut self, node_id: NodeId, port_id: NodePortId) -> Result<()> {
        let mut node = self.node_view_mut();
        node.port_state().set_port_internal(node_id, port_id)
    }

    fn internal_connections(&self) -> impl Iterator<Item = (&NodeId, &NodePortId, &ChannelId)> {
        self.node_view().into_port_state().internal_connections()
    }

    fn connection_views(&self) -> impl Iterator<Item = Result<PortConnectionDataView<'_>>> {
        self.node_view().into_port_state().connection_views()
    }

    fn connection_views_by_kind(
        &self,
        kind: NodePortKind,
    ) -> impl Iterator<Item = Result<PortConnectionDataView<'_>>> {
        self.node_view()
            .into_port_state()
            .connection_views_by_kind(kind)
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

    fn borrow(&self) -> impl EdgeQueryView;
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

    fn borrow(&self) -> impl EdgeQueryView {
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

    fn borrow(&self) -> impl ChannelQueryView;
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

    fn borrow(&self) -> impl ChannelQueryView {
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

pub trait ChannelLifecycleApi: ChannelApi + ChannelUiApi + NodePortApi {
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
        let to_disconnect: Vec<_> = self
            .internal_connections()
            .filter(|(_, _, channel_id)| **channel_id == id)
            .map(|(node_id, port_id, _)| (*node_id, *port_id))
            .collect();

        for (node_id, port_id) in to_disconnect {
            self.disconnect_port(node_id, port_id, id)?;
        }

        let removed_channel = ChannelApi::remove(self, id);
        let removed_channel_ui = ChannelUiApi::remove(self, id);
        if removed_channel.is_none() || removed_channel_ui.is_none() {
            bail!("channel {id} does not exist");
        }

        Ok(())
    }
}

impl<T> ChannelLifecycleApi for T where T: ChannelApi + ChannelUiApi + NodePortApi {}

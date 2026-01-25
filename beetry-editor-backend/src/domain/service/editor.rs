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
            EditorRepositoryView, EditorRepositoryViewMut, NodeRepositoryFacadeConcept,
            NodeRepositoryFacadeView, UiRepositoryFacadeConcept,
        },
        service::{
            channel::{ChannelBorrowMutApi, ChannelService},
            edge::{EdgeBorrowMutApi, EdgeService},
            node::{self, NodeBorrowMutApi, NodeService},
        },
    },
    edge::EdgeQueryApi,
    node::{SpecByNodeIdQuery, SpecByNodeIdQueryApi, SpecBySpecIdQuery, SpecBySpecIdQueryApi},
    ui::{
        ChannelUiBorrowApi, ChannelUiBorrowMutApi, ChannelUiQueryApi, NodeUiBorrowApi,
        NodeUiBorrowMutApi, NodeUiQueryApi,
    },
};
use anyhow::Result;
use beetry_editor_types::{
    id::{ChannelId, EdgeId, NodeId},
    output::{
        channel::{ChannelConfig, ChannelData},
        edge::NodeEdge,
        ui::{ChannelUiData, NodeUiData, Point},
    },
    spec::{channel::ChannelSpec, node::NodeSpec},
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

    pub fn node_api(&self) -> NodeBorrowApi<'_, NRF> {
        let EditorRepositoryView { node, .. } = self.repo.view();
        NodeBorrowApi::new(node.view(), &self.node_service)
    }

    pub fn node_api_mut(&mut self) -> NodeBorrowMutApi<'_, NRF, ER, CRF> {
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
}

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

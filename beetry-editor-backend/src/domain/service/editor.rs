use crate::{
    NodeSpecMap,
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
            ui::UiBorrowMutApi,
        },
        ui::UiBorrowApi,
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
    pub fn new(spec_plugins: NodeSpecMap) -> Self {
        Self {
            node_service: NodeService::new(),
            edge_service: EdgeService::new(),
            channel_service: ChannelService::new(),
            repo: EditorRepository::<NRF, ER, CRF, URF>::new(),
            spec_map: spec_plugins,
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

    pub fn edge_api(&self) -> EdgeBorrowApi<'_, ER> {
        let EditorRepositoryView { edge, .. } = self.repo.view();
        EdgeBorrowApi::new(edge, &self.edge_service)
    }

    pub fn edge_api_mut(&mut self) -> EdgeBorrowMutApi<'_, ER, NRF> {
        let EditorRepositoryViewMut { node, edge, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeView { nodes, specs, .. } = node.view();
        let tracker_api = node::TrackerApi::new(&self.node_service, nodes);
        let spec_api = node::SpecApi::new(specs, nodes);
        EdgeBorrowMutApi::new(edge, &mut self.edge_service, tracker_api, spec_api)
    }

    pub fn channel_api(&self) -> ChannelBorrowApi<'_, CRF> {
        let EditorRepositoryView { channel, .. } = self.repo.view();
        ChannelBorrowApi::new(channel.view())
    }

    pub fn channel_api_mut(&mut self) -> ChannelBorrowMutApi<'_, CRF> {
        let EditorRepositoryViewMut { channel, .. } = self.repo.view_mut();
        ChannelBorrowMutApi::new(channel.view_mut(), &mut self.channel_service)
    }

    pub fn ui_api_mut(&mut self) -> UiBorrowMutApi<'_, URF> {
        let EditorRepositoryViewMut { ui, .. } = self.repo.view_mut();
        UiBorrowMutApi::new(ui.view_mut())
    }

    pub fn ui_api(&self) -> UiBorrowApi<'_, URF> {
        let EditorRepositoryView { ui, .. } = self.repo.view();
        UiBorrowApi::new(ui.view(), &self.node_service)
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

    pub fn export_api(&self) -> ExportApi<'_, NRF, ER, CRF, URF> {
        ExportApi::new(
            self.channel_api(),
            self.node_api(),
            self.edge_api(),
            self.ui_api(),
        )
    }
}

use crate::domain::{
    repository::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, EditorRepository,
        EditorRepositoryViewMut, NodeRepositoryFacadeConcept, NodeRepositoryFacadeView,
        UiRepositoryFacadeConcept,
    },
    service::{
        channel::{ChannelService, ChannelServiceApi},
        edge::{EdgeService, EdgeServiceApi},
        node::{self, NodeService, NodeServiceApi},
        ui::UiServiceApi,
    },
};

pub struct EditorService<NRF, ER, CRF, URF> {
    node_service: NodeService,
    edge_service: EdgeService,
    channel_service: ChannelService,
    repo: EditorRepository<NRF, ER, CRF, URF>,
}

impl<NRF, ER, CRF, URF> Default for EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    fn default() -> Self {
        Self {
            node_service: NodeService::new(),
            edge_service: EdgeService::new(),
            channel_service: ChannelService::new(),
            repo: EditorRepository::<NRF, ER, CRF, URF>::new(),
        }
    }
}

impl<NRF, ER, CRF, URF> EditorService<NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn node_api(&mut self) -> NodeServiceApi<'_, NRF, ER, CRF> {
        let EditorRepositoryViewMut {
            node,
            edge,
            channel,
            ..
        } = self.repo.view_mut();

        NodeServiceApi::new(
            node.view_mut(),
            &mut self.node_service,
            edge,
            &mut self.edge_service,
            channel,
            &mut self.channel_service,
        )
    }

    pub fn edge_api(&mut self) -> EdgeServiceApi<'_, ER, NRF> {
        let EditorRepositoryViewMut { node, edge, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeView { nodes, specs, .. } = node.view();
        let tracker_api = node::TrackerServiceApi::new(&self.node_service, nodes);
        let spec_api = node::SpecServiceApi::new(specs, nodes);
        EdgeServiceApi::new(edge, &mut self.edge_service, tracker_api, spec_api)
    }

    pub fn channel_api(&mut self) -> ChannelServiceApi<'_, CRF> {
        let EditorRepositoryViewMut { channel, .. } = self.repo.view_mut();
        ChannelServiceApi::new(channel.view_mut(), &mut self.channel_service)
    }

    pub fn ui_api(&mut self) -> UiServiceApi<'_, URF> {
        let EditorRepositoryViewMut { ui, .. } = self.repo.view_mut();
        UiServiceApi::new(ui.view_mut())
    }
}

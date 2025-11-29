use crate::domain::{
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        NodeRepositoryFacadeConcept, ParamRepositoryConcept,
    },
    service::{
        channel::{ChannelService, ChannelServiceView},
        node::{NodeService, NodeServiceView},
    },
};

pub struct EditorService<NRF, ER, CR> {
    node_service: NodeService,
    channel_service: ChannelService,
    repo: EditorRepository<NRF, ER, CR>,
}

impl<NRF, ER, CR> EditorService<NRF, ER, CR>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelRepositoryConcept,
{
    pub fn new(repo: EditorRepository<NRF, ER, CR>) -> Self {
        Self {
            node_service: NodeService::new(),
            channel_service: ChannelService::new(),
            repo,
        }
    }

    pub fn node_view(&mut self) -> NodeServiceView<'_, '_, NRF, ER, CR> {
        NodeServiceView::new(&mut self.repo, &mut self.node_service)
    }

    pub fn channel_view(&mut self) -> ChannelServiceView<'_, '_, '_, NRF, ER, CR> {
        ChannelServiceView::new(
            &mut self.repo,
            &mut self.channel_service,
            &self.node_service,
        )
    }
}

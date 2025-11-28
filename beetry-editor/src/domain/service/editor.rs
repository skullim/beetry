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

pub struct EditorService<NRF, ER, CR, PR> {
    node_service: NodeService,
    channel_service: ChannelService,
    repo: EditorRepository<NRF, ER, CR, PR>,
}

impl<NRF, ER, CR, PR> EditorService<NRF, ER, CR, PR>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelRepositoryConcept,
    PR: ParamRepositoryConcept,
{
    pub fn new(repo: EditorRepository<NRF, ER, CR, PR>) -> Self {
        Self {
            node_service: NodeService::new(),
            channel_service: ChannelService::new(),
            repo,
        }
    }

    pub fn node_view(&mut self) -> NodeServiceView<'_, '_, NRF, ER, CR, PR> {
        NodeServiceView::new(&mut self.repo, &mut self.node_service)
    }

    pub fn channel_view(&mut self) -> ChannelServiceView<'_, '_, '_, NRF, ER, CR, PR> {
        ChannelServiceView::new(
            &mut self.repo,
            &mut self.channel_service,
            &self.node_service,
        )
    }
}

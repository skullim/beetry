use crate::domain::{
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        NodeRepositoryFacadeConcept,
    },
    service::{
        channel::{ChannelService, ChannelServiceView},
        edge::{EdgeService, EdgeServiceView},
        node::{AssignNodeId, IncrementalNodeIdAssigner, NodeService, NodeServiceView},
    },
};

pub struct EditorService<NRF, ER, CR, A> {
    node_service: NodeService<A>,
    edge_service: EdgeService,
    channel_service: ChannelService,
    repo: EditorRepository<NRF, ER, CR>,
}
impl<NRF, ER, CR> Default for EditorService<NRF, ER, CR, IncrementalNodeIdAssigner>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelRepositoryConcept,
{
    fn default() -> Self {
        Self {
            node_service: NodeService::<IncrementalNodeIdAssigner>::new(),
            edge_service: EdgeService::new(),
            channel_service: ChannelService::new(),
            repo: EditorRepository::<NRF, ER, CR>::new(),
        }
    }
}

impl<NRF, ER, CR, A> EditorService<NRF, ER, CR, A>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelRepositoryConcept,
    A: AssignNodeId,
{
    pub fn with_node_service(node_service: NodeService<A>) -> Self {
        Self {
            node_service,
            edge_service: EdgeService::new(),
            channel_service: ChannelService::new(),
            repo: EditorRepository::<NRF, ER, CR>::new(),
        }
    }

    pub fn node_view(&mut self) -> NodeServiceView<'_, '_, '_, NRF, ER, CR, A> {
        NodeServiceView::new(
            &mut self.repo,
            &mut self.node_service,
            &mut self.edge_service,
        )
    }

    pub fn edge_view(&mut self) -> EdgeServiceView<'_, '_, NRF, ER, CR> {
        EdgeServiceView::new(&mut self.repo, &mut self.edge_service)
    }

    pub fn channel_view(&mut self) -> ChannelServiceView<'_, '_, NRF, ER, CR> {
        ChannelServiceView::new(&mut self.repo, &mut self.channel_service)
    }
}

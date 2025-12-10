use crate::domain::{
    ports::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, EditorRepository,
        EditorRepositoryViewMut, NodeRepositoryFacadeConcept,
    },
    service::{
        channel::{ChannelService, ChannelServiceView},
        edge::{EdgeService, EdgeServiceView},
        node::{NodeService, NodeServiceView},
    },
};

pub struct EditorService<NRF, ER, CRF> {
    node_service: NodeService,
    edge_service: EdgeService,
    channel_service: ChannelService,
    repo: EditorRepository<NRF, ER, CRF>,
}

impl<NRF, ER, CRF> Default for EditorService<NRF, ER, CRF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    fn default() -> Self {
        Self {
            node_service: NodeService::new(),
            edge_service: EdgeService::new(),
            channel_service: ChannelService::new(),
            repo: EditorRepository::<NRF, ER, CRF>::new(),
        }
    }
}

impl<NRF, ER, CRF> EditorService<NRF, ER, CRF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    pub fn with_node_service(node_service: NodeService) -> Self {
        Self {
            node_service,
            edge_service: EdgeService::new(),
            channel_service: ChannelService::new(),
            repo: EditorRepository::<NRF, ER, CRF>::new(),
        }
    }

    pub fn node_view(&mut self) -> NodeServiceView<'_, '_, NRF> {
        NodeServiceView::new(self.repo.node_mut().view_mut(), &mut self.node_service)
    }

    pub fn edge_view(&mut self) -> EdgeServiceView<'_, ER, NRF> {
        let EditorRepositoryViewMut { node, edge, .. } = self.repo.view_mut();
        EdgeServiceView::new(edge, &mut self.edge_service, node.view())
    }

    pub fn channel_view(&mut self) -> ChannelServiceView<'_, CRF> {
        ChannelServiceView::new(
            self.repo.channel_mut().view_mut(),
            &mut self.channel_service,
        )
    }
}

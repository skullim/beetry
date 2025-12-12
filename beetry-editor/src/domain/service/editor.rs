use crate::domain::{
    repository::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, EditorRepository,
        EditorRepositoryViewMut, NodeRepositoryFacadeConcept, NodeRepositoryFacadeView,
        NodeRepositoryFacadeViewMut,
    },
    service::{
        channel::{self, ChannelService, ChannelServiceApi},
        edge::{EdgeService, EdgeServiceApi},
        node::{self, NodeService, NodeServiceApi},
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn node_api(&mut self) -> NodeServiceApi<'_, NRF, ER> {
        let EditorRepositoryViewMut { node, edge, .. } = self.repo.view_mut();

        NodeServiceApi::new(
            node.view_mut(),
            &mut self.node_service,
            edge,
            &mut self.edge_service,
        )
    }

    pub fn edge_api(&mut self) -> EdgeServiceApi<'_, ER, NRF> {
        let EditorRepositoryViewMut { node, edge, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeView { nodes, specs, .. } = node.view();
        let tracker_api = node::TrackerServiceApi::new(&self.node_service, nodes);
        let spec_api = node::SpecServiceApi::new(specs, nodes);
        EdgeServiceApi::new(edge, &mut self.edge_service, tracker_api, spec_api)
    }

    pub fn channel_api(&mut self) -> ChannelServiceApi<'_, CRF, NRF> {
        let EditorRepositoryViewMut { node, channel, .. } = self.repo.view_mut();
        let NodeRepositoryFacadeViewMut {
            nodes,
            specs,
            ports,
            ..
        } = node.view_mut();
        let tracker_api = node::TrackerServiceApi::new(&self.node_service, nodes);
        let spec_api = node::SpecServiceApi::new(specs, nodes);
        let ports_api = node::PortStateServiceApi::new(ports);
        let external_deps = channel::ExternalDeps::new(tracker_api, spec_api, ports_api);

        ChannelServiceApi::new(channel.view_mut(), &mut self.channel_service, external_deps)
    }
}

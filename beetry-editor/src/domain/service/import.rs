use crate::domain::{
    persistence::{EditorStorage, TreeStorage},
    repository::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, EditorRepository,
        EditorRepositoryViewMut, NodeRepositoryFacadeConcept,
    },
    service::{
        channel::ChannelService,
        edge::EdgeService,
        node::{LoadNodeApi, NodeService},
    },
};
use anyhow::Result;

pub struct ImportServiceApi<'a, NRF, ER, CRF> {
    node_service: &'a mut NodeService,
    edge_service: &'a mut EdgeService,
    channel_service: &'a mut ChannelService,
    repo: &'a mut EditorRepository<NRF, ER, CRF>,
}

impl<'a, NRF, ER, CRF> ImportServiceApi<'a, NRF, ER, CRF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    pub fn import_project(&mut self, data: EditorStorage) -> Result<()> {
        self.import_tree(data.tree)
    }

    pub fn import_tree(&mut self, tree: TreeStorage) -> Result<()> {
        let EditorRepositoryViewMut { node, channel, .. } = self.repo.view_mut();
        let node_view = node.view_mut();
        let mut load_api =
            LoadNodeApi::new(self.node_service, self.channel_service, node_view, channel);
        for spec_record in tree.node.specs {
            load_api.load_spec(spec_record)?;
        }

        for node_record in tree.node.nodes {
            load_api.load_node(node_record)?;
        }

        todo!("load edges and channels")
    }
}

//@todo implement in next release
struct SubtreeImporter;

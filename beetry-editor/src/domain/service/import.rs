use crate::domain::{
    persistence::{EditorStore, TreeStore, UiElementStore},
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
    pub fn import_project(&mut self, store: EditorStore) -> Result<()> {
        self.import_tree(store.tree)?;
        self.import_ui(store.ui_elements)
    }

    pub fn import_tree(&mut self, tree: TreeStore) -> Result<()> {
        let EditorRepositoryViewMut { node, channel, .. } = self.repo.view_mut();
        let node_view = node.view_mut();
        let mut load_api =
            LoadNodeApi::new(self.node_service, self.channel_service, node_view, channel);
        for spec_record in tree.graph.specs {
            load_api.load_spec(spec_record)?;
        }

        for node_record in tree.graph.nodes {
            load_api.load_node(node_record)?;
        }

        todo!("load edges and channels")
    }

    pub fn import_ui(&mut self, ui: UiElementStore) -> Result<()> {
        todo!()
    }
}

//@todo implement in next release
struct SubtreeImporter;

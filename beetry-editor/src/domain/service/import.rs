use crate::{
    SpecPlugins,
    domain::{
        persistence::{EditorStateStore, MaybeValidTree, UiElementStore},
        repository::{
            ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, EditorRepository,
            EditorRepositoryViewMut, NodeRepositoryFacadeConcept, UiRepositoryFacadeConcept,
        },
        service::{
            channel::ChannelService,
            edge::EdgeService,
            node::{LoadNodeApi, NodeService},
        },
    },
};
use anyhow::Result;

pub struct ImportApi<'a, NRF, ER, CRF, URF> {
    node_service: &'a mut NodeService,
    edge_service: &'a mut EdgeService,
    channel_service: &'a mut ChannelService,
    repo: &'a mut EditorRepository<NRF, ER, CRF, URF>,
    specs: &'a SpecPlugins,
}

impl<'a, NRF, ER, CRF, URF> ImportApi<'a, NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    pub fn import_project(&mut self, store: EditorStateStore) -> Result<()> {
        self.import_tree(store.tree)?;
        self.import_ui(store.ui_elements)
    }

    pub fn import_tree(&mut self, MaybeValidTree(mut tree): MaybeValidTree) -> Result<()> {
        let EditorRepositoryViewMut { node, channel, .. } = self.repo.view_mut();
        let node_view = node.view_mut();
        let mut load_api =
            LoadNodeApi::new(self.node_service, self.channel_service, node_view, channel);
        for spec_record in tree.node.specs.values() {
            //@todo need to get the plugins here to load fully the specs
            //load_api.load_spec(spec_record)?;
        }

        for record in tree.node.nodes.into_records() {
            let param_value = tree.parameter.take(&record.id);
            let port_state = tree.ports.take(&record.id);
            load_api.load_node(record, param_value, port_state)?;
        }

        todo!("load edges and channels")
    }

    pub fn import_ui(&mut self, ui: UiElementStore) -> Result<()> {
        todo!()
    }
}

//@todo implement in next release
struct SubtreeImporter;

use crate::{
    NodeSpecMap,
    domain::{
        channel::LoadChannelApi,
        edge::EdgeBorrowMutApi,
        node,
        repository::{
            ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, EditorRepository,
            EditorRepositoryViewMut, NodeRepositoryFacadeConcept, NodeRepositoryFacadeView,
            UiRepositoryFacadeConcept,
        },
        service::{
            channel::ChannelService,
            edge::EdgeService,
            node::{LoadNodeApi, NodeService},
        },
        ui::UiBorrowMutApi,
    },
};
use anyhow::Result;
use beetry_editor_types::{
    output::edge::NodeEdge,
    persistence::{EditorStateStore, MaybeValidTree, UiElementStore},
};

pub struct ImportApi<'a, NRF, ER, CRF, URF> {
    node_service: &'a mut NodeService,
    edge_service: &'a mut EdgeService,
    channel_service: &'a mut ChannelService,
    repo: &'a mut EditorRepository<NRF, ER, CRF, URF>,
    spec_map: &'a NodeSpecMap,
}

impl<'a, NRF, ER, CRF, URF> ImportApi<'a, NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    pub(super) fn new(
        node_service: &'a mut NodeService,
        edge_service: &'a mut EdgeService,
        channel_service: &'a mut ChannelService,
        repo: &'a mut EditorRepository<NRF, ER, CRF, URF>,
        spec_map: &'a NodeSpecMap,
    ) -> Self {
        Self {
            node_service,
            edge_service,
            channel_service,
            repo,
            spec_map,
        }
    }

    pub fn import_project(&mut self, store: EditorStateStore) -> Result<()> {
        self.reset_editor_state();
        self.import_tree(store.tree)?;
        self.import_ui(store.ui_elements)
    }

    pub fn import_tree(&mut self, MaybeValidTree(mut tree): MaybeValidTree) -> Result<()> {
        self.reset_editor_state();

        let EditorRepositoryViewMut {
            node,
            channel,
            edge,
            ..
        } = self.repo.view_mut();
        {
            let mut load_channel_api =
                LoadChannelApi::new(channel.view_mut(), self.channel_service);
            for record in tree.channel.specs.into_records() {
                load_channel_api.load_spec(record)?;
            }

            for record in tree.channel.channels.into_records() {
                load_channel_api.load_channel(record)?;
            }
        }

        let mut edges = vec![];
        {
            let node_view = node.view_mut();
            let mut load_node_api = LoadNodeApi::new(self.node_service, node_view);
            for (spec_id, spec_key) in tree.node.specs.iter() {
                let spec = self.spec_map.spec(spec_key)?;
                load_node_api.load_spec(*spec_id, spec.clone())?;
            }

            for record in tree.node.nodes.into_records() {
                let edge_iter = record.value.children().map(|to| NodeEdge {
                    from: record.id,
                    to: *to,
                });
                edges.extend(edge_iter);

                let param_value = tree.parameter.take(&record.id);
                let port_state = tree.ports.take(&record.id);
                load_node_api.load_node(record, param_value, port_state)?;
            }
        }
        let NodeRepositoryFacadeView { nodes, specs, .. } = node.view();
        let tracker_api = node::TrackerApi::new(self.node_service, nodes);
        let spec_api = node::SpecApi::new(specs, nodes);
        let mut edge_mut_api: EdgeBorrowMutApi<'_, ER, NRF> =
            EdgeBorrowMutApi::new(edge, self.edge_service, tracker_api, spec_api);
        for edge in edges {
            edge_mut_api.create(edge)?;
        }
        Ok(())
    }

    pub fn import_ui(&mut self, ui: UiElementStore) -> Result<()> {
        let mut ui_mut_api = UiBorrowMutApi::new(self.repo.ui_mut().view_mut());
        {
            let mut node_mut_api = ui_mut_api.node();
            for node in ui.nodes {
                node_mut_api.create(node.id, node.data)?;
            }

            let mut channel_mut_api = ui_mut_api.channel();
            for channel in ui.channels {
                channel_mut_api.create(channel.id, channel.data)?;
            }
        }
        Ok(())
    }

    fn reset_editor_state(&mut self) {
        *self.node_service = NodeService::new();
        *self.edge_service = EdgeService::new();
        *self.channel_service = ChannelService::new();
        *self.repo = EditorRepository::default();
    }
}

//@todo implement in next release
#[allow(unused)]
struct SubtreeImporter;

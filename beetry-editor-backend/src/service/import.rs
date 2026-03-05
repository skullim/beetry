use crate::{
    NodeSpecMap,
    channel::LoadChannelView,
    edge::EdgeViewMut,
    node,
    repository::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, EditorRepository,
        EditorRepositoryViewMut, NodeRepositoryFacadeConcept, NodeRepositoryFacadeView,
        UiRepositoryFacadeConcept,
    },
    service::{
        channel::ChannelService,
        edge::EdgeService,
        node::{LoadNodeView, NodeService},
    },
    ui::{ChannelUiViewMut, NodeUiViewMut, PortConnectionUiStateViewMut},
};
use anyhow::Result;
use beetry_editor_types::{
    output::edge::NodeEdge,
    persistence::{EditorStateStore, MaybeValidTree, UiElementStore},
};

pub struct ImportViewMut<'a, NRF, ER, CRF, URF> {
    node_service: &'a mut NodeService,
    edge_service: &'a mut EdgeService,
    channel_service: &'a mut ChannelService,
    repo: &'a mut EditorRepository<NRF, ER, CRF, URF>,
    spec_map: &'a NodeSpecMap,
}

impl<'a, NRF, ER, CRF, URF> ImportViewMut<'a, NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    pub(crate) fn new(
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
            let mut load_channel_view =
                LoadChannelView::new(channel.view_mut(), self.channel_service);
            for record in tree.channel.specs.into_records() {
                load_channel_view.load_spec(record)?;
            }

            for record in tree.channel.channels.into_records() {
                load_channel_view.load_channel(record)?;
            }
        }

        let mut edges = vec![];
        {
            let node_view = node.view_mut();
            let mut load_node_view = LoadNodeView::new(self.node_service, node_view);
            for (spec_id, spec_key) in tree.node.specs.iter() {
                let spec = self.spec_map.spec(spec_key)?;
                load_node_view.load_spec(*spec_id, spec.clone())?;
            }

            for record in tree.node.nodes.into_records() {
                let edge_iter = record.value.children().map(|to| NodeEdge {
                    from: record.id,
                    to: *to,
                });
                edges.extend(edge_iter);

                let param_value = tree.parameter.take(&record.id);
                let ports_state = tree.port.take_state(&record.id);
                load_node_view.load_node(record, param_value, ports_state)?;
            }
            load_node_view.load_port_connections(tree.port.take_connections())?;
        }
        let NodeRepositoryFacadeView { nodes, specs, .. } = node.view();
        let tracker_view = node::TrackerView::new(self.node_service, nodes);
        let spec_view = node::SpecView::new(specs, nodes);
        let mut edge_mut_api: EdgeViewMut<'_, ER, NRF> =
            EdgeViewMut::new(edge, self.edge_service, tracker_view, spec_view);
        for edge in edges {
            edge_mut_api.create(edge)?;
        }
        Ok(())
    }

    pub fn import_ui(&mut self, ui: UiElementStore) -> Result<()> {
        let UiElementStore {
            nodes,
            channels,
            port_connections,
        } = ui;

        let mut node_mut_api = NodeUiViewMut::new(self.repo.ui_mut().view_mut().node);
        for node in nodes {
            node_mut_api.create(node.id, node.data)?;
        }

        let mut channel_mut_api = ChannelUiViewMut::new(self.repo.ui_mut().view_mut().channel);
        for channel in channels {
            channel_mut_api.create(channel.id, channel.data)?;
        }

        let EditorRepositoryViewMut { node, ui, .. } = self.repo.view_mut();
        let mut port_conn_ui_api = PortConnectionUiStateViewMut::new(
            ui.view_mut().port_connection,
            node.view().port_connections,
        );
        for conn in port_connections {
            port_conn_ui_api.create(conn.id, conn.data)?;
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

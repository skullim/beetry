use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use beetry_editor_types::{
    id::{ChannelId, NodeId},
    persistence::{channel, editor, node, parameter, port, tree, ui},
    spec::node::NodeSpecKey,
};

use crate::{
    api::{NodeTrackerQuery, ParameterValueQuery},
    channel::ChannelQueryView,
    node::{PortConnectionQuery, PortStateQuery, SpecByNodeIdQuery},
    repository::PortConnectionUiRepository,
    service::{edge::EdgeQueryView, node::NodeView},
    ui::{ChannelUiQuery, NodeUiQuery, NodeUiQueryProcessor},
};

pub struct ExportView<'a, NSQ, EQ, CQ, NUQ, CUQ>
where
    NSQ: SpecByNodeIdQuery,
    EQ: EdgeQueryView,
    CQ: ChannelQueryView,
    NUQ: NodeUiQuery,
    CUQ: ChannelUiQuery,
{
    channel_api: CQ,
    node_api: NodeView<'a>,
    node_spec_query_api: NSQ,
    edge_api: EQ,
    node_ui_api: NUQ,
    channel_ui_api: CUQ,
    port_connection_ui_repo: &'a PortConnectionUiRepository,
}

impl<'a, NSQ, EQ, CQ, NUQ, CUQ> ExportView<'a, NSQ, EQ, CQ, NUQ, CUQ>
where
    NSQ: SpecByNodeIdQuery,
    EQ: EdgeQueryView,
    CQ: ChannelQueryView,
    NUQ: NodeUiQuery,
    CUQ: ChannelUiQuery,
{
    pub fn new(
        channel_api: CQ,
        node_api: NodeView<'a>,
        node_spec_query_api: NSQ,
        edge_api: EQ,
        node_ui_api: NUQ,
        channel_ui_api: CUQ,
        port_connection_ui_api: &'a PortConnectionUiRepository,
    ) -> Self {
        Self {
            channel_api,
            node_api,
            node_spec_query_api,
            edge_api,
            node_ui_api,
            channel_ui_api,
            port_connection_ui_repo: port_connection_ui_api,
        }
    }

    /// Project can be exported at any time, even if some parts of the tree are
    /// not yet connected
    pub fn export_project(&self) -> Result<editor::StateStore> {
        let tracker = self.node_api.tracker();

        let nodes: Vec<_> = tracker.nodes().copied().collect();
        let node_store = self.export_node_store(&nodes)?;
        let port_store = self.export_port_store(&nodes);
        let param_store = self.export_parameter_store(&nodes)?;

        let channels: Vec<_> = self.channel_api.channels().copied().collect();
        let channel_store = self.export_channel_store(&channels)?;

        let tree_store = tree::MaybeValid(tree::Store::new(
            node_store,
            port_store,
            param_store,
            channel_store,
        ));

        Ok(editor::StateStore {
            tree: tree_store,
            ui_elements: self.export_ui_elements(),
        })
    }

    pub fn export_valid_tree(&self) -> Result<tree::ValidTreeStore> {
        let nodes_to_export: Vec<_> = self.node_api.tracker().nodes().copied().collect();
        let node_store = self.export_node_store(&nodes_to_export)?;
        let param_store = self.export_parameter_store(&nodes_to_export)?;
        let port_store = self.export_port_store(&nodes_to_export);

        let channels_to_export: Vec<_> = self.channel_api.channels().copied().collect();
        let channel_store = self.export_channel_store(&channels_to_export)?;

        let tree = tree::Store::new(node_store, port_store, param_store, channel_store);
        tree::ValidTreeStore::try_from(tree)
            .map_err(|errors| anyhow!("attempted to export invalid tree, details: {errors}"))
    }

    fn export_node_store(&self, nodes: &[NodeId]) -> Result<node::Store> {
        let specs = {
            let tracker_api = self.node_api.tracker();
            let spec_api = self.node_api.spec();

            nodes
                .iter()
                .map(|id| {
                    let spec_id = tracker_api
                        .spec_id(*id)
                        .with_context(|| anyhow!("expected spec id for node {id}"))?;

                    Ok((
                        spec_id,
                        NodeSpecKey::new(
                            spec_api.name_by_spec_id(spec_id)?.clone(),
                            spec_api.kind_by_spec_id(spec_id)?,
                        ),
                    ))
                })
                .collect::<Result<node::SpecStore>>()?
        };

        let nodes = {
            let tracker_api = self.node_api.tracker();
            nodes
                .iter()
                .copied()
                .map(|id| {
                    let mut children: Vec<_> = self.edge_api.children_of(id).copied().collect();
                    let query_processor = NodeUiQueryProcessor::new(&self.node_ui_api);
                    query_processor
                        .sort_nodes(&mut children, |l, r| l.position.x.total_cmp(&r.position.x))?;

                    Ok((
                        id,
                        node::RecordValue::new(
                            tracker_api
                                .spec_id(id)
                                .with_context(|| anyhow!("expected spec id for node {id}"))?,
                            children.into_iter(),
                        ),
                    ))
                })
                .collect::<Result<node::RecordStore>>()?
        };
        Ok(node::Store { specs, nodes })
    }

    fn export_parameter_store(&self, nodes: &[NodeId]) -> Result<parameter::Store> {
        let parameter_api = self.node_api.parameter();
        let spec_api = self.node_api.spec();
        let nodes = nodes.iter().filter(|id| spec_api.params(**id).is_ok());
        let store = nodes
            .copied()
            .map(|id| {
                Ok((
                    id,
                    parameter::Values {
                        params: parameter_api.parameters(id)?.clone(),
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>>>()
            .with_context(|| anyhow!("failed to export parameters"))?;
        Ok(parameter::Store::new(store))
    }

    fn export_port_store(&self, nodes: &[NodeId]) -> port::Store {
        let ports_api = self.node_api.port_state();
        let mut state_records = Vec::with_capacity(nodes.len());
        for node_id in nodes.iter().copied() {
            let ports_state: port::StateMap = self
                .node_spec_query_api
                .ports(node_id)
                .iter()
                .flat_map(|ports| ports.ids().copied())
                .filter_map(|port_id| {
                    ports_api
                        .state(node_id, port_id)
                        .ok()
                        .map(|state| (port_id, state.clone()))
                })
                .collect();
            state_records.push(port::StateRecord::new(node_id, ports_state));
        }
        let port_connection_view = self.node_api.port_connection_query();
        let connections: Vec<_> = port_connection_view.all_connections().collect();
        port::Store::new(state_records, connections)
    }

    fn export_channel_store(&self, channels: &[ChannelId]) -> Result<channel::Store> {
        let specs = channels
            .iter()
            .copied()
            .map(|id| {
                Ok((
                    self.channel_api.spec_id(id)?,
                    self.channel_api.spec(id)?.clone(),
                ))
            })
            .collect::<Result<channel::SpecStore>>()?;

        let channels = channels
            .iter()
            .copied()
            .map(|id| Ok((id, self.channel_api.data(id)?.clone())))
            .collect::<Result<channel::DataStore>>()?;

        Ok(channel::Store { specs, channels })
    }

    fn export_ui_elements(&self) -> ui::Store {
        let channels: Vec<_> = {
            self.channel_ui_api
                .iter()
                .map(|(id, data)| ui::ChannelRecord {
                    id: *id,
                    data: data.clone(),
                })
                .collect()
        };
        let nodes: Vec<_> = {
            self.node_ui_api
                .iter()
                .map(|(id, data)| ui::NodeRecord {
                    id: *id,
                    data: data.clone(),
                })
                .collect()
        };
        let port_connections: Vec<_> = self
            .port_connection_ui_repo
            .iter()
            .map(|(id, data)| ui::PortConnectionRecord {
                id: *id,
                data: data.clone(),
            })
            .collect();
        ui::Store::new(nodes, channels, port_connections)
    }
}

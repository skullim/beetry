use anyhow::{Context, Result, anyhow, bail};
use bon::Builder;
use itertools::Itertools;
use mitsein::iter1::FromIterator1;

use std::collections::{BTreeSet, HashMap, HashSet};
use tracing::warn;

use crate::{
    channel::ChannelQueryApi,
    domain::{
        repository::NodeRepositoryFacadeConcept,
        service::{edge::EdgeQueryApi, node::NodeView},
    },
    node::SpecByNodeIdQueryApi,
    ui::{ChannelUiQueryApi, NodeUiQueryApi, NodeUiQueryProcessor},
};
use beetry_editor_types::{
    id::{ChannelId, NodeId, NodePortId},
    persistence::{
        ChannelDataStore, ChannelSpecStore, ChannelStore, ChannelUiRecord, EditorStateStore,
        MaybeValidTree, NodeRecordStore, NodeRecordValue, NodeSpecStore, NodeStore, NodeUiRecord,
        ParameterStore, ParameterValues, PortConnectionCollection, PortConnectionRecord,
        PortStateStore, TreeStore, UiElementStore, ValidTree,
    },
    spec::node::NodeSpecKey,
};

#[derive(Debug, Builder)]
pub struct TreeValidationResult {
    #[builder(default)]
    pub missing_root: bool,
    pub child_free_non_leaf_node: Option<NodeId>,
    pub unconnected_port: Option<(NodeId, NodePortId)>,
    // nodes that are not connected to root
    #[builder(default)]
    pub unconnected_nodes: HashSet<NodeId>,
    #[builder(default)]
    pub unconnected_channels: HashSet<ChannelId>,
}

impl TreeValidationResult {
    pub fn is_tree_valid(&self) -> bool {
        self.child_free_non_leaf_node.is_none() && self.unconnected_port.is_none()
    }
}

pub struct ExportView<'a, NRF, NSQ, EQ, CQ, NUQ, CUQ>
where
    NRF: NodeRepositoryFacadeConcept,
    NSQ: SpecByNodeIdQueryApi,
    EQ: EdgeQueryApi,
    CQ: ChannelQueryApi,
    NUQ: NodeUiQueryApi,
    CUQ: ChannelUiQueryApi,
{
    channel_api: CQ,
    node_api: NodeView<'a, NRF>,
    node_spec_query_api: NSQ,
    edge_api: EQ,
    node_ui_api: NUQ,
    channel_ui_api: CUQ,
}

impl<'a, NRF, NSQ, EQ, CQ, NUQ, CUQ> ExportView<'a, NRF, NSQ, EQ, CQ, NUQ, CUQ>
where
    NRF: NodeRepositoryFacadeConcept,
    NSQ: SpecByNodeIdQueryApi,
    EQ: EdgeQueryApi,
    CQ: ChannelQueryApi,
    NUQ: NodeUiQueryApi,
    CUQ: ChannelUiQueryApi,
{
    pub fn new(
        channel_api: CQ,
        node_api: NodeView<'a, NRF>,
        node_spec_query_api: NSQ,
        edge_api: EQ,
        node_ui_api: NUQ,
        channel_ui_api: CUQ,
    ) -> Self {
        Self {
            channel_api,
            node_api,
            node_spec_query_api,
            edge_api,
            node_ui_api,
            channel_ui_api,
        }
    }

    /// Project can be exported at any time, even if some parts of the tree are not yet connected
    pub fn export_project(&mut self) -> Result<EditorStateStore> {
        let tracker = self.node_api.tracker();

        let nodes: Vec<_> = tracker.nodes().copied().collect();
        let node_store = self.export_node_store(&nodes)?;
        let port_store = self.export_port_store(&nodes)?;
        let param_store = self.export_parameter_store(&nodes)?;

        let channels: Vec<_> = self.channel_api.channels().copied().collect();
        let channel_store = self.export_channel_store(&channels)?;

        let tree_store = MaybeValidTree(TreeStore::new(
            node_store,
            port_store,
            param_store,
            channel_store,
        ));

        Ok(EditorStateStore {
            tree: tree_store,
            ui_elements: self.export_ui_elements(),
        })
    }

    /// Tree can be exported only if tree is valid and fully connected
    pub fn export_valid_tree(&mut self) -> Result<ValidTree> {
        let validation = self.validate_tree();
        if !validation.is_tree_valid() {
            bail!("attempted to export invalid tree, details: {validation:?}");
        }

        let nodes_to_export: Vec<_> = {
            if !validation.unconnected_nodes.is_empty() {
                warn!(
                    "removing detected unconnected nodes {:?} from the export",
                    validation.unconnected_nodes
                );
            }
            let tracker = self.node_api.tracker();
            tracker
                .nodes()
                .filter(|id| !validation.unconnected_nodes.contains(*id))
                .copied()
                .collect()
        };

        let channels_to_export: Vec<_> = {
            if !validation.unconnected_channels.is_empty() {
                warn!(
                    "removing detected unconnected channels {:?} from the export",
                    validation.unconnected_channels
                );
            }
            let channel_id_iter = self.channel_api.channels();
            channel_id_iter
                .filter(|id| !validation.unconnected_channels.contains(*id))
                .copied()
                .collect()
        };
        let channel_store = self.export_channel_store(&channels_to_export)?;

        let node_store = self.export_node_store(&nodes_to_export)?;
        let param_store = self.export_parameter_store(&nodes_to_export)?;
        let port_store = self.export_port_store(&nodes_to_export)?;

        let tree = TreeStore::new(node_store, port_store, param_store, channel_store);
        Ok(ValidTree::new(tree))
    }

    fn export_node_store(&mut self, nodes: &[NodeId]) -> Result<NodeStore> {
        let specs = {
            let tracker_api = self.node_api.tracker();
            let spec_ids = nodes
                .iter()
                //@todo handle unwrap
                .map(|id| {
                    tracker_api
                        .spec_id(*id)
                        .with_context(|| anyhow!("expected spec id for node {id}"))
                        .unwrap()
                })
                .unique();

            let spec_api = self.node_api.spec();
            spec_ids
                .map(|spec_id| {
                    Ok((
                        spec_id,
                        NodeSpecKey::new(
                            spec_api.name_by_spec_id(spec_id)?.clone(),
                            spec_api.kind_by_spec_id(spec_id)?,
                        ),
                    ))
                })
                .collect::<Result<NodeSpecStore>>()?
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
                        NodeRecordValue::new(
                            tracker_api
                                .spec_id(id)
                                .with_context(|| anyhow!("expected spec id for node {id}"))?,
                            children.into_iter(),
                        ),
                    ))
                })
                .collect::<Result<NodeRecordStore>>()?
        };
        Ok(NodeStore { specs, nodes })
    }

    fn export_parameter_store(&mut self, nodes: &[NodeId]) -> Result<ParameterStore> {
        let parameter_api = self.node_api.parameter();
        let spec_api = self.node_api.spec();
        let nodes = nodes.iter().filter(|id| spec_api.params(**id).is_ok());
        let store = nodes
            .copied()
            .map(|id| {
                Ok((
                    id,
                    ParameterValues {
                        params: parameter_api.parameters(id)?.clone(),
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>>>()
            .with_context(|| anyhow!("failed to export parameters"))?;
        Ok(ParameterStore::new(store))
    }

    fn export_port_store(&mut self, nodes: &[NodeId]) -> Result<PortStateStore> {
        let ports_api = self.node_api.port_state();
        Ok(nodes
            .iter()
            .copied()
            .filter_map(|id| {
                PortConnectionCollection::try_from_iter(
                    ports_api
                        .node_conns(id)
                        .map(|(id, conn)| PortConnectionRecord::new(*id, conn.clone())),
                )
                // filter collections that actually have any ports
                .ok()
                .map(|collection| (id, collection))
            })
            .collect())
    }

    fn export_channel_store(&mut self, channels: &[ChannelId]) -> Result<ChannelStore> {
        let specs = channels
            .iter()
            .copied()
            .map(|id| {
                Ok((
                    self.channel_api.spec_id(id)?,
                    self.channel_api.spec(id)?.clone(),
                ))
            })
            .collect::<Result<ChannelSpecStore>>()?;

        let channels = channels
            .iter()
            .copied()
            .map(|id| Ok((id, self.channel_api.data(id)?.clone())))
            .collect::<Result<ChannelDataStore>>()?;

        Ok(ChannelStore { specs, channels })
    }

    fn export_ui_elements(&mut self) -> UiElementStore {
        let channels: Vec<_> = {
            self.channel_ui_api
                .iter()
                .map(|(id, data)| ChannelUiRecord {
                    id: *id,
                    data: data.clone(),
                })
                .collect()
        };
        let nodes: Vec<_> = {
            self.node_ui_api
                .iter()
                .map(|(id, data)| NodeUiRecord {
                    id: *id,
                    data: data.clone(),
                })
                .collect()
        };
        UiElementStore::new(nodes, channels)
    }

    //@todo check if channel ports are connected to nodes that are part of the graph
    /// Validation rules:
    /// 0. Root node exists
    /// 1. Each node is connected to root
    /// 2. All except leaf nodes have at least (or most for decorator) 1 child. Decorator having maximum one child is guaranteed at node connection API.
    /// 3. Each node port is not in Unconnected state
    /// 4. Optional: Gather list of unconnected channels (if any)
    fn validate_tree(&self) -> TreeValidationResult {
        let (root_id, leaf_nodes): (_, HashSet<_>) = {
            let tracker = self.node_api.tracker();
            match tracker.root_id() {
                Ok(root_id) => (root_id, tracker.leaf_nodes().copied().collect()),
                Err(_) => {
                    return TreeValidationResult::builder().missing_root(true).build();
                }
            }
        };

        let mut valid_nodes = HashSet::new();
        let mut to_visit = BTreeSet::from_iter(std::iter::once(root_id));

        while let Some(parent) = to_visit.pop_first() {
            let mut children = self.edge_api.children_of(parent).copied().peekable();
            if children.peek().is_none() {
                return TreeValidationResult::builder()
                    .child_free_non_leaf_node(parent)
                    .build();
            }
            for child in children {
                if !leaf_nodes.contains(&child) {
                    to_visit.insert(child);
                    continue;
                }

                if let Some(ports_spec) = self
                    .node_spec_query_api
                    .spec(child)
                    .expect("leaf node must have a spec")
                    .ports()
                {
                    for port_id in ports_spec.ids() {
                        if self.node_api.port_state().state(child, *port_id).is_err() {
                            return TreeValidationResult::builder()
                                .unconnected_port((child, *port_id))
                                .build();
                        }
                    }
                }
                valid_nodes.insert(child);
            }
            valid_nodes.insert(parent);
        }

        let unconnected_nodes = self
            .node_api
            .tracker()
            .nodes()
            .filter(|id| !valid_nodes.contains(*id))
            .copied()
            .collect();
        TreeValidationResult::builder()
            .unconnected_nodes(unconnected_nodes)
            .build()
    }
}

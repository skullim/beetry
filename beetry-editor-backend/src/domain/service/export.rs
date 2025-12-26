use anyhow::{Context, Result, anyhow, bail};
use bon::Builder;
use itertools::Itertools;

use std::collections::{BTreeSet, HashMap, HashSet};
use tracing::warn;

use beetry_editor_types::{
    id::{ChannelId, NodeId, NodePortId},
    persistence::{
        ChannelDataStore, ChannelSpecStore, ChannelStore, ChannelUiRecord, EditorStateStore,
        MaybeValidTree, NodePortState, NodePortStore, NodeRecordStore, NodeRecordValue,
        NodeSpecStore, NodeStore, NodeUiRecord, ParameterValue, ParameterValueStore, TreeStore,
        UiElementStore, ValidTree,
    },
    spec::node::NodeSpecKey,
};

use crate::domain::{
    repository::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, NodeRepositoryFacadeConcept,
        UiRepositoryFacadeConcept,
    },
    service::{
        channel::ChannelBorrowApi, edge::EdgeBorrowApi, node::NodeBorrowApi, ui::UiBorrowApi,
    },
};

#[derive(Debug, Builder)]
pub struct TreeValidationResult {
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

pub struct ExportApi<'a, NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    channel_api: ChannelBorrowApi<'a, CRF>,
    node_api: NodeBorrowApi<'a, NRF>,
    edge_api: EdgeBorrowApi<'a, ER>,
    ui_api: UiBorrowApi<'a, URF>,
}

impl<'a, NRF, ER, CRF, URF> ExportApi<'a, NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    pub fn new(
        channel_api: ChannelBorrowApi<'a, CRF>,
        node_api: NodeBorrowApi<'a, NRF>,
        edge_api: EdgeBorrowApi<'a, ER>,
        ui_api: UiBorrowApi<'a, URF>,
    ) -> Self {
        Self {
            channel_api,
            node_api,
            edge_api,
            ui_api,
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
                .map(|id| tracker_api.spec_id(*id).unwrap())
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
                    self.ui_api.node().sort_children(&mut children, |l, r| {
                        l.position.origin.x.total_cmp(&r.position.origin.x)
                    })?;

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

    fn export_parameter_store(&mut self, nodes: &[NodeId]) -> Result<ParameterValueStore> {
        let parameter_api = self.node_api.parameter();
        let spec_api = self.node_api.spec();
        let nodes = nodes.iter().filter(|id| {
            if let Ok(param) = spec_api.params(**id)
                && !param.defs.is_empty()
            {
                true
            } else {
                false
            }
        });
        let store = nodes
            .copied()
            .map(|id| {
                Ok((
                    id,
                    ParameterValue {
                        params: parameter_api.parameters(id)?.clone(),
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>>>()
            .with_context(|| anyhow!("failed to export parameters"))?;
        Ok(ParameterValueStore::new(store))
    }

    fn export_port_store(&mut self, nodes: &[NodeId]) -> Result<NodePortStore> {
        let ports_api = self.node_api.port_state();
        let port_state_iter = nodes.iter().copied().map(|id| {
            (
                id,
                NodePortState::new(
                    ports_api
                        .port_iter(id)
                        .map(|(id, conn)| (*id, conn.clone())),
                ),
            )
        });
        Ok(NodePortStore::new(port_state_iter))
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
            let channel_api = self.ui_api.channel();
            channel_api
                .iter()
                .map(|(id, data)| ChannelUiRecord {
                    id: *id,
                    data: data.clone(),
                })
                .collect()
        };
        let nodes: Vec<_> = {
            let node_api = self.ui_api.node();
            node_api
                .iter()
                .map(|(id, data)| NodeUiRecord {
                    id: *id,
                    data: data.clone(),
                })
                .collect()
        };
        UiElementStore::new(nodes, channels)
    }

    /// Preconditions:
    /// 1. There is one and only root node
    ///
    /// Validation rules:
    /// 1. Each node is connected to root
    /// 2. All except leaf nodes have at least (or most for decorator) 1 child. Decorator having maximum one child is guaranteed at node connection API.
    /// 3. Each node port is not in Unconnected state
    /// 4. Optional: Gather list of unconnected channels (if any)
    fn validate_tree(&self) -> TreeValidationResult {
        let (root_id, leaf_nodes): (_, HashSet<_>) = {
            let tracker = self.node_api.tracker();
            (
                tracker
                    .root_id()
                    .context("precondition that root exists not met")
                    .unwrap(),
                tracker.leaf_nodes().copied().collect(),
            )
        };

        let mut valid_nodes = HashSet::new();
        let mut to_visit = BTreeSet::from_iter(std::iter::once(root_id));

        while let Some(parent) = to_visit.pop_first() {
            let children: Vec<_> = self.edge_api.children_of(parent).copied().collect();
            if children.is_empty() {
                return TreeValidationResult::builder()
                    .child_free_non_leaf_node(parent)
                    .build();
            }
            for child in children {
                if leaf_nodes.contains(&child) {
                    if let Some(unconnected) = self
                        .node_api
                        .port_state()
                        .port_iter(child)
                        .find(|(_, conn)| !conn.is_valid())
                    {
                        return TreeValidationResult::builder()
                            .unconnected_port((child, *unconnected.0))
                            .build();
                    } else {
                        valid_nodes.insert(child);
                    }
                } else {
                    to_visit.insert(child);
                }
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

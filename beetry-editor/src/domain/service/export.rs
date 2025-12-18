use anyhow::{Context, Result, anyhow, bail};
use beetry_plugin_types::node::LeafKind;
use beetry_reconstruction_types::{
    channel::{ChannelSnapshot2, ChannelSnapshotMap},
    node::{ControlSnapshot, LeafSnapshot, NodeSnapshot, NodeSnapshotData, RootSnapshot},
    tree::TreeSnapshot,
};
use bon::Builder;

use itertools::izip;
use std::collections::{BTreeSet, HashMap, HashSet};
use tracing::warn;

use crate::domain::{
    models::{
        ChannelId, ChannelUiData, NodeId, NodeKind, NodePortConnection, NodePortId, NodePortKind,
        NodeSpecKey,
    },
    persistence::{
        ChannelRecord, ChannelSpecRecord, ChannelStore, ChannelUiRecord, EditorStateStore,
        MaybeValidTree, NodePortState, NodePortStore, NodeRecord, NodeSpecRecord, NodeStore,
        NodeUiRecord, ParameterValue, ParameterValueStore, TreeStore, UiElementStore, ValidTree,
    },
    repository::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, NodeRepositoryFacadeConcept,
        UiRepositoryFacadeConcept,
    },
    service::{
        channel::ChannelServiceApi, edge::EdgeServiceApi, node::NodeServiceApi, ui::UiServiceApi,
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

pub struct ExportServiceApi<'a, NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
    channel_api: ChannelServiceApi<'a, CRF>,
    node_api: NodeServiceApi<'a, NRF, ER, CRF>,
    edge_api: EdgeServiceApi<'a, ER, NRF>,
    ui_api: UiServiceApi<'a, URF>,
}

impl<'a, NRF, ER, CRF, URF> ExportServiceApi<'a, NRF, ER, CRF, URF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
    URF: UiRepositoryFacadeConcept,
{
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
                .filter(|id| validation.unconnected_nodes.contains(*id))
                .copied()
                .collect()
        };

        let node_store = self.export_node_store(&nodes_to_export)?;
        let param_store = self.export_parameter_store(&nodes_to_export)?;
        let port_store = self.export_port_store(&nodes_to_export)?;

        let channels_to_export: Vec<_> = {
            if !validation.unconnected_channels.is_empty() {
                warn!(
                    "removing detected unconnected channels {:?} from the export",
                    validation.unconnected_channels
                );
            }
            let channel_id_iter = self.channel_api.channels();
            channel_id_iter
                .filter(|id| validation.unconnected_channels.contains(*id))
                .copied()
                .collect()
        };
        let channel_store = self.export_channel_store(&channels_to_export)?;

        let tree = TreeStore::new(node_store, port_store, param_store, channel_store);
        Ok(ValidTree::new(tree))
    }

    fn export_node_store(&mut self, nodes: &[NodeId]) -> Result<NodeStore> {
        let specs = {
            let spec_api = self.node_api.spec();
            nodes
                .iter()
                .copied()
                .map(|id| {
                    Ok(NodeSpecRecord {
                        id,
                        key: NodeSpecKey::new(spec_api.name(id)?.clone(), spec_api.kind(id)?),
                    })
                })
                .collect::<Result<Vec<_>>>()
        }?;

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

                    Ok(NodeRecord::new(
                        id,
                        tracker_api
                            .spec_id(id)
                            .with_context(|| anyhow!("expected spec id for node {id}"))?,
                        children.into_iter(),
                    ))
                })
                .collect::<Result<Vec<NodeRecord>>>()?
        };
        Ok(NodeStore { specs, nodes })
    }

    fn export_parameter_store(&mut self, nodes: &[NodeId]) -> Result<ParameterValueStore> {
        let parameters_api = self.node_api.parameters();
        let store = nodes
            .iter()
            .copied()
            .map(|id| {
                Ok((
                    id,
                    ParameterValue {
                        params: parameters_api.parameters(id)?.clone(),
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;
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
                Ok(ChannelSpecRecord {
                    id: self.channel_api.spec_id(id)?,
                    spec: self.channel_api.spec(id)?.clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let records = channels
            .iter()
            .copied()
            .map(|id| {
                Ok(ChannelRecord {
                    id,
                    data: self.channel_api.data(id)?.clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(ChannelStore {
            specs,
            channels: records,
        })
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
    //@todo long term should be shared reference, but PortConnectionServiceApi requires exclusive reference
    fn validate_tree(&mut self) -> TreeValidationResult {
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
            let children: Vec<_> = self.edge_api.children_of(parent).collect();
            if children.is_empty() {
                return TreeValidationResult::builder()
                    .child_free_non_leaf_node(parent)
                    .build();
            }
            for child in children {
                if leaf_nodes.contains(child) {
                    if let Some(unconnected) = self
                        .node_api
                        .port_state()
                        .port_iter(*child)
                        .find(|(_, conn)| !conn.is_valid())
                    {
                        return TreeValidationResult::builder()
                            .unconnected_port((*child, *unconnected.0))
                            .build();
                    }
                    continue;
                } else {
                    to_visit.insert(*child);
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

    //@todo move to reconstruction crate
    pub fn snapshot_from(&mut self, tree: ValidTree) -> Result<TreeSnapshot> {
        let tree = tree.into_inner();
        let root = self.export_root()?;
        Ok(TreeSnapshot::builder()
            .root(root)
            .channels(self.export_channels()?)
            .build()?)
    }

    fn export_channels(&mut self) -> Result<ChannelSnapshotMap> {
        let ids: Vec<_> = self.channel_api.channels().copied().collect();
        let spec = ids
            .iter()
            .map(|id| self.channel_api.spec(*id))
            .collect::<Result<Vec<_>>>()?;
        let config = ids
            .iter()
            .map(|id| self.channel_api.config(*id))
            .collect::<Result<Vec<_>>>()?;

        let map = izip!(ids, spec, config)
            .map(|(id, spec, meta)| (id, ChannelSnapshot2::new(spec.clone(), meta.clone())))
            .collect::<HashMap<_, _>>();

        todo!()
    }

    fn export_root(&mut self) -> Result<RootSnapshot> {
        let root_id = self.node_api.tracker().root_id()?;

        let child_id = self
            .edge_api
            .children_of(root_id)
            .next()
            .copied()
            .ok_or_else(|| anyhow!("root has no child"))?;
        let child = self.export_node(child_id)?;
        Ok(RootSnapshot::new(child))
    }

    fn export_node(&mut self, id: NodeId) -> Result<NodeSnapshot> {
        let kind: NodeKind = self.node_api.spec().kind(id)?;
        match kind {
            NodeKind::Root => unreachable!(),
            NodeKind::Control => {
                let child_ids: Vec<_> = self.edge_api.children_of(id).copied().collect();
                //@todo: child_ids have to be sorted based on the x coordinate (increasing) to determine the proper children order
                let mut children = Vec::with_capacity(child_ids.len());
                for child_id in child_ids {
                    children.push(self.export_node(child_id)?);
                }

                Ok(NodeSnapshot::builder()
                    .name(self.node_api.spec().name(id)?.clone())
                    .data(NodeSnapshotData::Control(ControlSnapshot::new(children)?))
                    .build())
            }
            NodeKind::Action => self.export_leaf(id, LeafKind::Action),
            NodeKind::Condition => self.export_leaf(id, LeafKind::Condition),

            NodeKind::Decorator => {
                unimplemented!()
            }
        }
    }

    fn export_leaf(&mut self, id: NodeId, kind: LeafKind) -> Result<NodeSnapshot> {
        let mut senders = vec![];
        let mut receivers = vec![];

        let ports_spec = {
            let spec_api = self.node_api.spec();
            spec_api.ports(id)?.clone()
        };
        let port_state_api = self.node_api.port_state();

        for port_id in ports_spec.ids() {
            let spec = ports_spec.spec(*port_id)?;
            let conn = port_state_api.state(id, *port_id)?;
            if let NodePortConnection::Internal(connected) = conn {
                if connected.is_empty() {
                    bail!("unconnected internal port ({port_id}) of node {id}");
                }
                match spec.kind {
                    NodePortKind::Sender => {
                        senders.extend(connected);
                    }
                    NodePortKind::Receiver => {
                        receivers.extend(connected);
                    }
                }
            }
        }

        //@todo add external senders/receivers
        let leaf_snapshot = LeafSnapshot::builder()
            .kind(kind)
            .receivers(receivers)
            .senders(senders)
            .build();

        let name = self.node_api.spec().name(id)?.clone();
        let params = self.node_api.parameters().parameters(id)?.clone();
        Ok(NodeSnapshot::builder()
            .name(name)
            .data(leaf_snapshot)
            .parameters(params)
            .build())
    }
}

use std::{
    collections::{BTreeSet, HashMap},
    io::Read,
    path::Path,
};

use anyhow::{Result, anyhow};
use beetry_core::MessageHash;
use beetry_serde::{
    de::{
        node::{ControlSnapshot, LeafSnapshot, NodeSnapshot, NodeSnapshotData, RootSnapshot},
        tree::TreeSnapshot,
    },
    ser::node::LeafSchema,
};
use beetry_serialization::{Deserializer, JsonDeserializer};
use dioxus_logger::tracing::debug;
use serde::{Deserialize, Serialize};

use crate::{
    definitions::{NodeEdge, NodeId},
    ui::{self, channel, edge},
};

#[derive(Serialize, Deserialize)]
pub struct ProjectData {
    pub version: u32,
    pub tree: TreeSnapshot,
    editor: EditorMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct EditorMetadata {
    pub(crate) nodes: ui::NodeMap,
    pub(crate) edges: Vec<NodeEdge>,
    pub(crate) last_id: NodeId,
    pub(crate) channel_tracker: channel::Tracker,
}

impl ProjectData {
    pub(crate) fn export(
        nodes: &ui::NodeMap,
        last_id: NodeId,
        edge_tracker: &edge::Tracker,
        channel_tracker: &channel::Tracker,
    ) -> Result<Self> {
        let (id, root) = nodes
            .iter()
            .find(|(_, node)| node.kind == ui::NodeKind::Root)
            .ok_or_else(|| anyhow!("root has to exist in nodes map"))?;
        let root_snapshot = Self::export_root(root, *id, nodes, edge_tracker, channel_tracker)?;
        let tree_snapshot = TreeSnapshot::builder()
            .root(root_snapshot)
            .channels(
                channel_tracker
                    .channels()
                    .into_iter()
                    .map(|(k, v)| (k, v.snapshot))
                    .collect(),
            )
            .build()?;
        Ok(Self {
            version: 1,
            tree: tree_snapshot,
            editor: EditorMetadata {
                nodes: nodes.clone(),
                edges: edge_tracker.edges().clone(),
                last_id,
                channel_tracker: channel_tracker.clone(),
            },
        })
    }

    fn export_root(
        root: &ui::Node,
        root_id: NodeId,
        nodes: &ui::NodeMap,
        edge_tracker: &edge::Tracker,
        channel_tracker: &channel::Tracker,
    ) -> Result<RootSnapshot> {
        if root.kind == ui::NodeKind::Root {
            let child_id = edge_tracker
                .children_of(&root_id)
                .ok_or_else(|| anyhow!("root is not connected to any child"))?
                .first()
                .ok_or_else(|| anyhow!("root is not connected to any child"))?;
            let child_node = nodes.get(child_id).unwrap();
            let child =
                Self::export_node(child_node, *child_id, nodes, edge_tracker, channel_tracker)?;
            return Ok(RootSnapshot::new(child));
        }
        Err(anyhow!("expected root node got {:?}", root.kind))
    }

    fn export_node(
        node: &ui::Node,
        node_id: NodeId,
        nodes: &ui::NodeMap,
        edge_tracker: &edge::Tracker,
        channel_tracker: &channel::Tracker,
    ) -> Result<NodeSnapshot> {
        match &node.kind {
            ui::NodeKind::Control { params_schema: _ } => {
                let children_id = edge_tracker
                    .children_of(&node_id)
                    .ok_or_else(|| anyhow!("control node must have at least one child"))?;
                let mut children = vec![];
                for id in children_id {
                    let child_node = nodes.get(id).unwrap();
                    children.push(Self::export_node(
                        child_node,
                        *id,
                        nodes,
                        edge_tracker,
                        channel_tracker,
                    )?);
                }
                Ok(NodeSnapshot::builder()
                    .name(node.name.clone())
                    .data(NodeSnapshotData::Control(ControlSnapshot::new(children)?))
                    .build())
            }

            ui::NodeKind::Leaf {
                schema,
                external_receivers,
            } => {
                Self::validate_node_connections(
                    node_id,
                    &node.name.0,
                    schema,
                    channel_tracker,
                    external_receivers,
                )?;

                let receivers_read = channel_tracker.receivers();
                let receivers = receivers_read
                    .get(&node_id)
                    .map(|receivers| receivers.iter().cloned());
                let senders_read = channel_tracker.senders();
                let senders = senders_read
                    .get(&node_id)
                    .map(|senders| senders.iter().cloned());

                let leaf_snapshot = LeafSnapshot::builder()
                    .kind(*schema.kind())
                    .maybe_receivers(receivers)
                    .maybe_senders(senders)
                    .ext_receivers(external_receivers.iter().cloned().collect())
                    .build();

                Ok(NodeSnapshot::builder()
                    .name(node.name.clone())
                    .data(leaf_snapshot)
                    .parameters(node.selected_params.clone())
                    .build())
            }

            _ => {
                unreachable!()
            }
        }
    }

    pub(crate) fn import(path: &Path) -> Result<EditorMetadata> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("import path should contain extension"))?;
        if !matches!(ext, "json") {
            return Err(anyhow!("unsupported extension: {ext:?}"));
        }

        let mut file = std::fs::File::open(path)?;
        let mut content_buffer = String::new();
        file.read_to_string(&mut content_buffer)?;
        let data: Self = JsonDeserializer::deserialize(&content_buffer)?;
        Ok(data.editor)
    }

    // @todo: might add additional validation method to make sure that the nodes that the senders are connected to are connected to the graph
    /// Validate that all required senders and receivers for a node are properly connected
    fn validate_node_connections(
        node_id: NodeId,
        name: &str,
        schema: &LeafSchema,
        channel_tracker: &channel::Tracker,
        external_receivers: &BTreeSet<MessageHash>,
    ) -> Result<()> {
        debug!("Validating connections for node {node_id} ({})", name);

        let expected_receivers: HashMap<MessageHash, String> = schema
            .receivers()
            .iter()
            .map(|recv| (*recv.hash(), recv.desc().clone()))
            .collect();

        let expected_senders: HashMap<MessageHash, String> = schema
            .senders()
            .iter()
            .map(|send| (*send.hash(), send.desc().clone()))
            .collect();

        debug!("Expected receivers: {expected_receivers:?}");
        debug!("Expected senders: {expected_senders:?}");

        let mut unconnected_receivers = expected_receivers;
        let mut unconnected_senders = expected_senders;

        let channels = channel_tracker.channels();

        if let Some(node_receivers) = channel_tracker.receivers().get(&node_id) {
            debug!("Node has receiver channels: {node_receivers:?}");
            for channel_id in node_receivers.iter() {
                if let Some(element) = channels.get(channel_id) {
                    let snapshot = element.snapshot.clone();
                    let msg_hash = snapshot.spec().msg_hash();
                    if unconnected_receivers.remove(msg_hash).is_some() {
                        debug!(
                            "✓ Connected receiver for message: {}",
                            snapshot.spec().as_str()
                        );
                    } else {
                        debug!(
                            "⚠ Found unexpected receiver channel for message: {}",
                            snapshot.spec().as_str()
                        );
                    }
                }
            }
        }

        if let Some(node_senders) = channel_tracker.senders().get(&node_id) {
            debug!("Node has sender channels: {node_senders:?}");
            for channel_id in node_senders.iter() {
                if let Some(element) = channels.get(channel_id) {
                    let snapshot = element.snapshot.clone();
                    let msg_hash = snapshot.spec().msg_hash();
                    if unconnected_senders.remove(msg_hash).is_some() {
                        debug!(
                            "✓ Connected sender for message: {}",
                            snapshot.spec().as_str()
                        );
                    } else {
                        debug!(
                            "⚠ Found unexpected sender channel for message: {}",
                            snapshot.spec().as_str()
                        );
                    }
                }
            }
        }

        for external in external_receivers {
            if let Some(desc_str) = unconnected_receivers.remove(external) {
                debug!("✓ External receiver for message: {}", desc_str);
            }
        }

        if !unconnected_receivers.is_empty() {
            let missing_receivers: Vec<String> = unconnected_receivers.values().cloned().collect();
            return Err(anyhow!(
                "Node '{name}' (id: {node_id}) has unconnected receivers: {}",
                missing_receivers.join(", ")
            ));
        }

        if !unconnected_senders.is_empty() {
            let missing_senders: Vec<String> = unconnected_senders.values().cloned().collect();
            return Err(anyhow!(
                "Node '{name}' (id: {node_id}) has unconnected senders: {}",
                missing_senders.join(", ")
            ));
        }

        debug!("✓ All connections validated for node {node_id}");
        Ok(())
    }
}

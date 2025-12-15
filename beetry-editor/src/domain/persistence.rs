use anyhow::Result;
use std::collections::HashMap;

use beetry_serde::{
    de::{
        channel::{ChannelId, ChannelImplKind, SenderReceiverCount},
        parameter,
    },
    ser::channel::ChannelSpec,
};

use crate::{
    definitions::{NodeEdge, NodeId},
    domain::{
        models::{
            ChannelPosition, ChannelSpecId, EdgeId, NodePortConnection, NodePortId, NodePosition,
            NodeSpec, NodeSpecId,
        },
        repository::{ChannelDataRepositoryConcept, NodeRepositoryFacadeConcept},
    },
};

pub struct EditorData {
    pub tree: TreeData,
    pub positions: UiElementPositions,
}

pub struct TreeData {
    pub node_specs: Vec<NodeSpec>,
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
    pub channel_metadata: Vec<ChannelSpecEntry>,
    pub channels: Vec<ChannelDataEntry>,
}

pub struct NodeSpecEntry {
    pub id: NodeSpecId,
    pub spec: NodeSpec,
}

pub struct NodeData {
    pub id: NodeId,
    pub spec_id: NodeSpecId,
    pub ports_data: Vec<(NodePortId, NodeChannelPortData)>,
    pub parameters: Option<parameter::Parameters>,
}

pub struct NodeChannelPortData {
    pub kind: NodePortConnection,
}

pub struct EdgeData {
    pub id: EdgeId,
    pub node_edge: NodeEdge,
}

pub struct ChannelSpecEntry {
    pub id: ChannelSpecId,
    pub spec: ChannelSpec,
}

pub struct ChannelDataEntry {
    pub id: ChannelId,
    pub spec_id: ChannelSpecId,
    pub count: SenderReceiverCount,
    pub capacity: usize,
    pub kind: ChannelImplKind,
}

pub struct UiElementPositions {
    pub nodes: HashMap<NodeId, NodePosition>,
    pub channels: HashMap<ChannelId, ChannelPosition>,
}

// Repository <-> Storage impl

use crate::domain::repository::NodeRepositoryConcept;

pub struct NodeRepositoryFacadeStorage;

impl NodeRepositoryFacadeStorage {
    fn serialize(
        facade: &impl NodeRepositoryFacadeConcept,
    ) -> Result<(Vec<NodeData>, Vec<NodeSpecEntry>)> {
        let view = facade.view();
        let node_ids = view.nodes.nodes();
        todo!()
    }
}

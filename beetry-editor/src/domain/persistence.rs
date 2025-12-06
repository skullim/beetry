use std::collections::HashMap;

use beetry_plugin::{ActionSpec, ConditionSpec};
use beetry_serde::{
    de::{
        channel::{ChannelId, ChannelImplKind, SenderReceiverCount},
        parameter,
    },
    ser::node::{ControlSpec, DecoratorSpec, RootSpec},
};

use crate::{
    definitions::{NodeEdge, NodeId},
    domain::models::{
        ChannelPosition, EdgeId, NodeChannelPortId, NodeKind, NodePortConnection, NodePosition,
    },
};

type MetadataId = usize;

pub struct EditorData {
    tree: TreeData,
    positions: UiElementPositions,
}

pub struct TreeData {
    node_metadata: Vec<NodeMetadata>,
    nodes: Vec<NodeData>,
    edges: Vec<EdgeData>,
    channels: Vec<ChannelData>,
}

pub enum NodeSpec {
    Root(RootSpec),
    Control(ControlSpec),
    Condition(ConditionSpec),
    Action(ActionSpec),
    Decorator(DecoratorSpec),
}

pub struct NodeMetadata {
    id: MetadataId,
    kind: NodeKind,
    spec: NodeSpec,
    port_ids: Vec<NodeChannelPortId>,
}

pub struct NodeChannelPortData {
    kind: NodePortConnection,
}

pub struct NodeData {
    id: NodeId,
    metadata_id: MetadataId,
    ports_data: Vec<(NodeChannelPortId, NodeChannelPortData)>,
    parameters: Option<parameter::Parameters>,
}

pub struct EdgeData {
    id: EdgeId,
    node_edge: NodeEdge,
}

pub struct ChannelData {
    id: ChannelId,
    count: SenderReceiverCount,
    capacity: usize,
    kind: ChannelImplKind,
}

pub struct UiElementPositions {
    nodes: HashMap<NodeId, NodePosition>,
    channels: HashMap<ChannelId, ChannelPosition>,
}

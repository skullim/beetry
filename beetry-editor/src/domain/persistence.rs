use anyhow::{Result, anyhow};
use derive_more::From;
use std::collections::HashMap;

use beetry_plugin::{ActionSpec, ConditionSpec};
use beetry_serde::{
    de::{
        channel::{ChannelId, ChannelImplKind, SenderReceiverCount},
        parameter,
    },
    ser::{
        channel::ChannelSpec,
        node::{ControlSpec, DecoratorSpec, RootSpec},
    },
};

use crate::{
    definitions::{NodeEdge, NodeId},
    domain::{
        models::{
            ChannelPosition, EdgeId, NodeChannelPortId, NodeKind, NodePortConnection, NodePosition,
        },
        ports::{
            ChannelRepositoryConcept, ConditionNodeRepository, NodeRepositoryFacade,
            NodeRepositoryFacadeConcept, RootNodeRepository,
        },
    },
};

type NodeMetadataId = usize;
type ChannelMetadataId = usize;

pub struct EditorData {
    pub tree: TreeData,
    pub positions: UiElementPositions,
}

pub struct TreeData {
    pub node_metadata: Vec<NodeMetadata>,
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
    pub channel_metadata: Vec<ChannelMetadata>,
    pub channels: Vec<ChannelData>,
}

#[derive(Debug)]
pub enum NodeSpec {
    Root(RootSpec),
    Control(ControlSpec),
    Condition(ConditionSpec),
    Action(ActionSpec),
    Decorator(DecoratorSpec),
}

impl NodeSpec {
    pub fn root(&self) -> Result<&RootSpec> {
        if let Self::Root(spec) = self {
            return Ok(spec);
        }
        Err(anyhow!("no root spec found"))
    }
}

pub struct NodeMetadata {
    pub id: NodeMetadataId,
    pub kind: NodeKind,
    pub spec: NodeSpec,
    pub port_ids: Vec<NodeChannelPortId>,
}

pub struct NodeData {
    pub id: NodeId,
    pub metadata_id: NodeMetadataId,
    pub ports_data: Vec<(NodeChannelPortId, NodeChannelPortData)>,
    pub parameters: Option<parameter::Parameters>,
}

pub struct NodeChannelPortData {
    pub kind: NodePortConnection,
}

pub struct EdgeData {
    pub id: EdgeId,
    pub node_edge: NodeEdge,
}

pub struct ChannelMetadata {
    pub metadata_id: ChannelMetadataId,
    pub spec: ChannelSpec,
}

pub struct ChannelData {
    pub id: ChannelId,
    pub metadata_id: ChannelMetadataId,
    pub count: SenderReceiverCount,
    pub capacity: usize,
    pub kind: ChannelImplKind,
}

pub struct UiElementPositions {
    pub nodes: HashMap<NodeId, NodePosition>,
    pub channels: HashMap<ChannelId, ChannelPosition>,
}

// Repository <-> Storage impl

use crate::domain::ports::NodeRepositoryConcept;

//@todo make it into iter instead of Vec
pub trait NodeRepositoryFacadeStorageConcept {
    fn serialize(&self) -> Result<(Vec<NodeData>, Vec<NodeMetadata>)>;
    fn deserialize(data: Vec<NodeData>, meta: Vec<NodeMetadata>) -> Result<i32>; //Result<impl NodeRepositoryFacadeConcept>;
}

//@todo check if generic impl for all types possible?
impl NodeRepositoryFacadeStorageConcept for NodeRepositoryFacade {
    fn deserialize(data: Vec<NodeData>, meta: Vec<NodeMetadata>) -> Result<i32> {
        let spec_lookup: HashMap<_, _> = meta
            .into_iter()
            .map(|m| (m.id, (m.spec, m.kind, m.port_ids)))
            .collect();

        let mut root_repo = RootNodeRepository::default();
        let mut condition_repo = ConditionNodeRepository::default();

        // for node in data {
        //     let meta = spec_lookup
        //         .get(&node.metadata_id)
        //         .ok_or_else(|| anyhow!("failed to obtain metadata for node {}", node.id))?;
        //     let (spec, kind, ports) = meta;
        //     match kind {
        //         NodeKind::Root => {
        //             root_repo.create(node.id, spec.root()?)?;
        //         }
        //     }
        // }

        Ok(0)
    }

    fn serialize(&self) -> Result<(Vec<NodeData>, Vec<NodeMetadata>)> {
        todo!()
    }
}

pub trait LoadChannelRepository {
    fn load(
        data: Vec<ChannelData>,
        meta: Vec<ChannelMetadata>,
    ) -> Result<impl ChannelRepositoryConcept>;
}

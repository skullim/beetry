use std::collections::BTreeSet;

use anyhow::{Result, anyhow};
use beetry_channel::{AnyBoxReceiver, AnyBoxSender};
use beetry_core::{BoxNode, MessageHash, NonEmptyNodes};
use beetry_editor_types::{
    id::{ChannelId, NodeId, NodePortId},
    output::node::Parameters,
    spec::{
        message::MessageSpec,
        node::{LeafKind, NodeName},
    },
};
use bon::Builder;
use derive_more::From;
use getset::{CopyGetters, Getters};
use mitsein::iter1::FromIterator1;
use mitsein::vec1::Vec1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootSnapshot {
    pub child: NodeSnapshot,
}

impl RootSnapshot {
    pub fn new(child: NodeSnapshot) -> Self {
        Self { child }
    }

    pub fn into_child(self) -> NodeSnapshot {
        self.child
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct NodeSnapshot {
    #[builder(into)]
    pub name: NodeName,
    #[builder(into)]
    pub data: NodeSnapshotData,
    #[builder(default)]
    pub parameters: Parameters,
}

impl NodeSnapshot {
    pub fn take_parameters(&mut self) -> Parameters {
        std::mem::take(&mut self.parameters)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub enum NodeSnapshotData {
    Control(ControlSnapshot),
    Decorator(DecoratorSnapshot),
    Leaf(LeafSnapshot),
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
pub struct ControlSnapshot {
    #[get = "pub"]
    children: Vec1<NodeSnapshot>,
}

impl ControlSnapshot {
    pub fn new(children: impl IntoIterator<Item = NodeSnapshot>) -> Result<Self> {
        let children = Vec1::try_from_iter(children)
            .map_err(|_| anyhow!("received empty children iterator"))?;
        Ok(Self { children })
    }

    pub fn into_children_iter(self) -> impl IntoIterator<Item = NodeSnapshot> {
        self.children.into_iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoratorSnapshot {
    // box to prevent infinite type recursion
    child: Box<NodeSnapshot>,
}

impl DecoratorSnapshot {
    pub fn new(child: NodeSnapshot) -> Self {
        Self {
            child: Box::new(child),
        }
    }
}

impl From<DecoratorSnapshot> for NodeSnapshot {
    fn from(value: DecoratorSnapshot) -> Self {
        *value.child
    }
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, Getters, CopyGetters)]
pub struct LeafSnapshot {
    #[get_copy = "pub"]
    kind: LeafKind,
    //@todo all ports should have NodePortId, so they should rather be BTreeMap
    // this will allow to coexist more than one port of given message type
    #[get = "pub"]
    #[builder(default, with = <_>::from_iter)]
    receivers: BTreeSet<ChannelId>,
    #[get = "pub"]
    #[builder(default, with = <_>::from_iter)]
    senders: BTreeSet<ChannelId>,
    #[get = "pub"]
    #[builder(default)]
    ext_receivers: Vec<MessageHash>,
    #[get = "pub"]
    #[builder(default)]
    ext_senders: Vec<MessageHash>,
}

impl LeafSnapshot {
    pub fn take_receivers(&mut self) -> impl IntoIterator<Item = ChannelId> {
        std::mem::take(&mut self.receivers)
    }

    pub fn take_ext_receivers(&mut self) -> impl IntoIterator<Item = MessageHash> {
        std::mem::take(&mut self.ext_receivers)
    }

    pub fn take_senders(&mut self) -> impl IntoIterator<Item = ChannelId> {
        std::mem::take(&mut self.senders)
    }
}

pub type LeafReconstructionData = NodeReconstructionData<LeafMetadata>;
pub type ActionReconstructionData = LeafReconstructionData;
pub type ConditionReconstructionData = LeafReconstructionData;
pub type ControlReconstructionData = NodeReconstructionData<ControlMetadata>;
pub type DecoratorReconstructionData = NodeReconstructionData<DecoratorMetadata>;

#[derive(Debug, Builder)]
pub struct NodeReconstructionData<D> {
    pub inner: D,
    #[builder(default)]
    pub parameters: Parameters,
}

#[derive(Debug, Default, Builder)]
pub struct LeafMetadata {
    #[builder(default, into)]
    pub receivers: Vec<AnyBoxReceiver>,
    #[builder(default, into)]
    pub senders: Vec<AnyBoxSender>,
}

impl LeafMetadata {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct ControlMetadata {
    pub children: NonEmptyNodes,
}

impl ControlMetadata {
    pub fn new(children: NonEmptyNodes) -> Self {
        Self { children }
    }
}

pub struct DecoratorMetadata {
    pub child: BoxNode,
}

impl DecoratorMetadata {
    pub fn new(child: BoxNode) -> Self {
        Self { child }
    }
}

/// Provides basic information regarding external communication endpoints.
/// User should utilize it to provide missing endpoints such that the tree can
/// be reconstructed.
#[derive(Default, Builder)]
pub struct ExternalContextInfo {
    pub receivers: Vec<ExternalEndpointInfo>,
    pub senders: Vec<ExternalEndpointInfo>,
}

pub struct ExternalEndpointInfo {
    // not ideal as the client has to lookup the node in editor,
    // but otherwise nodes with the same name would be undistinguishable
    pub id: NodeId,
    pub name: NodeName,
    pub ports: Vec<ExternalPortInfo>,
}

pub struct ExternalPortInfo {
    pub id: NodePortId,
    pub message: MessageSpec,
}

use std::collections::BTreeSet;

use anyhow::{Result, anyhow};
use beetry_core::MessageHash;
use bon::Builder;
use derive_more::From;
use getset::{CopyGetters, Getters};
use mitsein::iter1::FromIterator1;
use mitsein::vec1::Vec1;
use serde::{Deserialize, Serialize};

use crate::de::channel::ChannelId;
use crate::de::parameter::Parameters;
use crate::ser::node::{LeafKind, NodeName};

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

#[derive(Debug, Clone, Builder, Serialize, Deserialize, Getters, CopyGetters)]
pub struct LeafSnapshot {
    #[get_copy = "pub"]
    kind: LeafKind,
    #[get = "pub"]
    #[builder(default, with = <_>::from_iter)]
    receivers: BTreeSet<ChannelId>,
    #[get = "pub"]
    #[builder(default, with = <_>::from_iter)]
    senders: BTreeSet<ChannelId>,
    #[get = "pub"]
    #[builder(default)]
    ext_receivers: Vec<MessageHash>,
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

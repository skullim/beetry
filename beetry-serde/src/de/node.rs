use std::collections::BTreeSet;

use anyhow::{Result, anyhow};
use beetry_core::MessageHash;
use bon::{Builder, builder};
use derive_getters::Getters;
use mitsein::{iter1::FromIterator1, vec1::Vec1};
use serde::{Deserialize, Serialize};

use crate::{
    de::{channel::ChannelId, parameter::SerializedParameters},
    ser::node::{LeafKind, NodeHash},
};

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
pub struct RootExport {
    child: NodeExport,
}

impl RootExport {
    pub fn new(child: NodeExport) -> Self {
        Self { child }
    }

    pub fn into_child(self) -> NodeExport {
        self.child
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeExport {
    Control(ControlExport),
    Leaf(LeafExport),
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
pub struct ControlExport {
    #[getter(copy)]
    kind: ControlKind,
    children: Vec1<Box<NodeExport>>,
}

impl ControlExport {
    pub fn new(
        kind: ControlKind,
        children: impl IntoIterator<Item = Box<NodeExport>>,
    ) -> Result<Self> {
        let children = Vec1::try_from_iter(children)
            .map_err(|_| anyhow!("received empty children iterator"))?;
        Ok(Self { kind, children })
    }

    pub fn into_children_iter(self) -> impl IntoIterator<Item = Box<NodeExport>> {
        self.children.into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ControlKind {
    Sequence,
    Fallback,
    Parallel,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, Getters)]
pub struct LeafExport {
    #[builder(into)]
    name: String,
    #[getter(copy)]
    kind: LeafKind,
    #[getter(copy)]
    hash: NodeHash,
    #[builder(default, with = <_>::from_iter)]
    receivers: BTreeSet<ChannelId>,
    #[builder(default, with = <_>::from_iter)]
    senders: BTreeSet<ChannelId>,
    #[builder(default)]
    external_receivers_export: Vec<MessageHash>,
    #[builder(default)]
    parameters: SerializedParameters,
}

impl LeafExport {
    pub fn take_receivers(&mut self) -> impl IntoIterator<Item = ChannelId> {
        std::mem::take(&mut self.receivers)
    }

    pub fn take_external_receivers_export(
        &mut self,
    ) -> Option<impl IntoIterator<Item = MessageHash>> {
        if self.external_receivers_export.is_empty() {
            return None;
        }
        Some(std::mem::take(&mut self.external_receivers_export))
    }

    pub fn take_senders(&mut self) -> impl IntoIterator<Item = ChannelId> {
        std::mem::take(&mut self.senders)
    }

    pub fn take_parameters(&mut self) -> SerializedParameters {
        std::mem::take(&mut self.parameters)
    }
}

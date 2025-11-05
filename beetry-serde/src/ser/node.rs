use std::{cmp::Ordering, collections::BTreeSet};

use bon::{Builder, builder};
use derive_getters::Getters;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::ser::node::leaf_schema_builder::SetKind;

use super::channel::MessageSpec;
use super::parameter;

#[derive(Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeName(pub String);

impl NodeName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

pub type LeafNodeSpec = NodeSpec<LeafSchema>;
pub type ActionNodeSpec = LeafNodeSpec;
pub type ConditionNodeSpec = LeafNodeSpec;
pub type ControlNodeSpec = NodeSpec<ControlSchema>;

#[derive(Debug, Builder, Clone, Eq, Getters, Serialize, Deserialize)]
pub struct NodeSpec<S> {
    pub name: NodeName,
    pub schema: S,
    #[builder(default)]
    pub params: parameter::Schema,
}

impl<S> PartialEq for NodeSpec<S> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<S> PartialOrd for NodeSpec<S>
where
    S: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S> Ord for NodeSpec<S>
where
    S: Eq + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Debug, Default)]
pub struct ControlSchema;

#[derive(Debug, Clone, PartialEq, Eq, Builder, Getters, Serialize, Deserialize)]
#[builder(finish_fn(vis = "pub(crate)"))]
pub struct LeafSchema {
    pub kind: LeafKind,
    #[builder(default, with = <_>::from_iter)]
    pub receivers: BTreeSet<MessageSpec>,
    #[builder(default, with = <_>::from_iter)]
    pub senders: BTreeSet<MessageSpec>,
}

pub struct ActionLeafSchema;
impl ActionLeafSchema {
    // cannot implement Default here as that would need to return ZST instead of LeafSchema
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> LeafSchema {
        Self::builder().build()
    }

    pub fn builder() -> LeafSchemaBuilder<SetKind> {
        LeafSchema::builder().kind(LeafKind::Action)
    }
}

pub struct ConditionLeafSchema;
impl ConditionLeafSchema {
    // cannot implement Default here as that would need to return ZST instead of LeafSchema
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> LeafSchema {
        Self::builder().build()
    }

    pub fn builder() -> LeafSchemaBuilder<SetKind> {
        LeafSchema::builder().kind(LeafKind::Condition)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LeafKind {
    Action,
    Condition,
}

use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashSet},
};

use bon::Builder;
use derive_getters::Getters;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::channel::MessageSpec;
use super::parameter::Schema;

#[derive(Debug, Clone, Getters)]
pub struct LeafSpecCollection {
    set: BTreeSet<LeafSpec>,
}

impl LeafSpecCollection {
    pub fn new(iter: impl IntoIterator<Item = LeafSpec>) -> Self {
        let mut set = BTreeSet::new();
        let mut seen_names = HashSet::new();
        for leaf in iter {
            if !seen_names.insert(leaf.name.clone()) {
                warn!(
                    "there exist at least one other leaf specification with name: {}, consider renaming",
                    leaf.name
                );
            }
            let leaf_name = leaf.name().clone();
            if !set.insert(leaf) {
                warn!(
                    "leaf specification collection already contains leaf with name {leaf_name}, skipping"
                );
            }
        }
        Self { set }
    }
}

#[derive(Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeName(pub String);

impl NodeName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

#[derive(Debug, Clone, Eq, Getters, Serialize, Deserialize)]
pub struct LeafSpec {
    pub name: NodeName,
    pub schema: LeafSchema,
}

impl LeafSpec {
    pub fn new(name: NodeName, schema: LeafSchema) -> Self {
        Self { name, schema }
    }
}

impl PartialEq for LeafSpec {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for LeafSpec {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LeafSpec {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Getters, Serialize, Deserialize)]

pub struct LeafSchema {
    #[getter(copy)]
    pub kind: LeafKind,
    #[builder(default, with = <_>::from_iter)]
    pub receivers: BTreeSet<MessageSpec>,
    #[builder(default, with = <_>::from_iter)]
    pub senders: BTreeSet<MessageSpec>,
    #[builder(default)]
    pub params: Schema,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LeafKind {
    Action,
    Condition,
}

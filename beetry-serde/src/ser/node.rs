use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashSet},
};

use bon::Builder;
use derive_getters::Getters;
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

            let leaf_hash = leaf.hash;
            if !set.insert(leaf) {
                warn!(
                    "leaf specification collection already contains leaf with hash {leaf_hash:?}, skipping"
                );
            }
        }
        Self { set }
    }
}

#[derive(Debug, Clone, Eq, Builder, Getters, Serialize, Deserialize)]
pub struct LeafSpec {
    #[builder(into)]
    name: String,
    // labels concrete node and its factory
    #[getter(copy)]
    hash: NodeHash,
    #[getter(copy)]
    kind: LeafKind,
    #[builder(default, with = <_>::from_iter)]
    receivers: BTreeSet<MessageSpec>,
    #[builder(default, with = <_>::from_iter)]
    senders: BTreeSet<MessageSpec>,
    #[builder(default)]
    params: Schema,
}

impl PartialEq for LeafSpec {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
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

/// Describes the hash of the node type (and not concrete node type instance)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeHash {
    hash: u64,
}

impl NodeHash {
    pub fn new(hash: u64) -> Self {
        Self { hash }
    }
}

pub trait NodeHashProvider {
    fn hash() -> NodeHash;
}

impl<T: type_hash::TypeHash> NodeHashProvider for T {
    fn hash() -> NodeHash {
        NodeHash::new(T::type_hash())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LeafKind {
    Action,
    Condition,
}

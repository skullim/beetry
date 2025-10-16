use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashSet},
};

use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::parameter::Schema;

#[derive(Debug, Clone, Getters)]
pub struct LeafDescriptionCollection {
    desc: BTreeSet<LeafDescription>,
}

impl LeafDescriptionCollection {
    pub fn new(iter: impl IntoIterator<Item = LeafDescription>) -> Self {
        let mut desc = BTreeSet::new();
        let mut seen_names = HashSet::new();
        for leaf in iter {
            if !seen_names.insert(leaf.name.clone()) {
                warn!(
                    "there exist at least one other leaf description with name: {}, consider renaming",
                    leaf.name
                );
            }

            let leaf_hash = leaf.hash;
            if !desc.insert(leaf) {
                warn!(
                    "leaf description collection already contains leaf with hash {leaf_hash:?}, skipping"
                );
            }
        }
        Self { desc }
    }
}

#[derive(Debug, Clone, Eq, Builder, Getters, Serialize, Deserialize)]
pub struct LeafDescription {
    #[builder(into)]
    name: String,
    // labels concrete node and its factory
    #[getter(copy)]
    hash: NodeHash,
    #[getter(copy)]
    kind: LeafKind,
    #[builder(default, with = <_>::from_iter)]
    receivers: BTreeSet<MessageDescription>,
    #[builder(default, with = <_>::from_iter)]
    senders: BTreeSet<MessageDescription>,
    #[builder(default)]
    params_schema: Schema,
}

impl PartialEq for LeafDescription {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl PartialOrd for LeafDescription {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LeafDescription {
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

#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, Deserialize)]
pub struct MessageDescription {
    desc: String,
    hash: MessageHash,
}

impl MessageDescription {
    pub fn new<T>(desc: impl Into<String>) -> Self
    where
        T: MessageHashProvider + 'static,
    {
        Self {
            desc: desc.into(),
            hash: T::hash(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.desc
    }
}

/// Marker trait for types that should be considered as message type
pub trait Message {}

pub trait MessageHashProvider {
    fn hash() -> MessageHash;
}

impl<T> MessageHashProvider for T
where
    T: Message + type_hash::TypeHash,
{
    fn hash() -> MessageHash {
        MessageHash::new(T::type_hash())
    }
}

pub trait MessageTypeProvider {
    fn as_str() -> &'static str;
}

//@todo: proc macro with optional string parameter would be cleaner, but requires new macro crate
impl<T: Message> MessageTypeProvider for T {
    fn as_str() -> &'static str {
        std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap_or(std::any::type_name::<T>())
    }
}

/// Describes the hash of the message type (and not concrete message type instance)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MessageHash {
    hash: u64,
}

impl MessageHash {
    pub fn new(hash: u64) -> Self {
        Self { hash }
    }
}

impl PartialOrd for MessageDescription {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MessageDescription {
    fn cmp(&self, other: &Self) -> Ordering {
        self.desc.cmp(&other.desc)
    }
}

#[derive(Debug, Clone, Getters, PartialEq, Serialize, Deserialize)]
pub struct ChannelDescription {
    // labels concrete channel and its factory
    msg_hash: MessageHash,
    msg_type_name: String,
}

impl ChannelDescription {
    pub fn new<T: MessageHashProvider + MessageTypeProvider>() -> Self {
        Self {
            msg_hash: T::hash(),
            msg_type_name: T::as_str().to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.msg_type_name
    }
}

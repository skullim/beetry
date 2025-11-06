use beetry_core::MessageHash;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, Deserialize)]
pub struct MessageSpec {
    desc: String,
    hash: MessageHash,
}

impl MessageSpec {
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

impl PartialOrd for MessageSpec {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MessageSpec {
    fn cmp(&self, other: &Self) -> Ordering {
        self.desc.cmp(&other.desc)
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
            .unwrap_or_else(|| std::any::type_name::<T>())
    }
}

#[derive(Debug, Clone, Getters, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelSpec {
    // labels concrete channel and its factory
    msg_hash: MessageHash,
    msg_type_name: String,
}

impl ChannelSpec {
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

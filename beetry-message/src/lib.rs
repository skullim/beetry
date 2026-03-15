use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Describes the hash of the message type (and not concrete message type
/// instance).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MessageHash {
    hash: u64,
}

impl MessageHash {
    #[must_use]
    pub fn new(hash: u64) -> Self {
        Self { hash }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, CopyGetters, Getters, Serialize, Deserialize)]
pub struct MessageSpec {
    #[get = "pub"]
    desc: String,
    #[get_copy = "pub"]
    hash: MessageHash,
}

impl MessageSpec {
    #[must_use]
    pub fn new<T>(desc: impl Into<String>) -> Self
    where
        T: MessageHashProvider + 'static,
    {
        Self {
            desc: desc.into(),
            hash: T::hash(),
        }
    }

    #[must_use]
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

/// Marker trait for types that should be considered as message type.
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

impl<T: Message> MessageTypeProvider for T {
    fn as_str() -> &'static str {
        std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap_or_else(|| std::any::type_name::<T>())
    }
}

//! Message typing primitives for Beetry.
//!
//! This crate is an internal Beetry implementation crate and is not considered
//! part of the public API. For public APIs, use the `beetry` crate.

use std::cmp::Ordering;

use getset::{CopyGetters, Getters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
/// Reexport macro needed to fully define Message to avoid users depending on
/// external crate
pub use type_hash;
/// Describes the hash of the message type (and not concrete message type
/// instance).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub struct MessageHash {
    hash: u64,
}

impl MessageHash {
    #[must_use]
    pub fn new(hash: u64) -> Self {
        Self { hash }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, CopyGetters, Getters, Serialize, Deserialize, JsonSchema,
)]
pub struct MessageSpec {
    #[get = "pub"]
    desc: String,
    #[get_copy = "pub"]
    hash: MessageHash,
}

impl MessageSpec {
    #[must_use]
    pub fn new<M>(desc: impl Into<String>) -> Self
    where
        M: Message,
    {
        Self {
            desc: desc.into(),
            hash: M::hash(),
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

/// Trait for types that should be considered as a message type.
pub trait Message: type_hash::TypeHash {
    fn hash() -> MessageHash {
        MessageHash::new(Self::type_hash())
    }

    fn as_str() -> &'static str {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap_or_else(|| std::any::type_name::<Self>())
    }
}

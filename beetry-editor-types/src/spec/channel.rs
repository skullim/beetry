use std::collections::HashMap;

use anyhow::{Result, anyhow};
use beetry_message::{MessageHash, MessageHashProvider, MessageTypeProvider};
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, CopyGetters, Getters)]
pub struct ChannelSpec {
    // labels concrete channel and its factory
    #[get_copy = "pub"]
    msg_hash: MessageHash,
    #[get = "pub"]
    msg_type_name: String,
}

impl ChannelSpec {
    #[must_use]
    pub fn new<T: MessageHashProvider + MessageTypeProvider>() -> Self {
        Self {
            msg_hash: T::hash(),
            msg_type_name: T::as_str().to_string(),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.msg_type_name
    }
}

#[derive(Debug, Clone)]
pub struct ChannelSpecMap {
    map: HashMap<MessageHash, ChannelSpec>,
}

impl FromIterator<(MessageHash, ChannelSpec)> for ChannelSpecMap {
    fn from_iter<T: IntoIterator<Item = (MessageHash, ChannelSpec)>>(iter: T) -> Self {
        Self {
            map: iter.into_iter().collect(),
        }
    }
}

impl ChannelSpecMap {
    pub fn spec(&self, key: &MessageHash) -> Result<&ChannelSpec> {
        self.map
            .get(key)
            .ok_or_else(|| anyhow!("failed to obtain channel spec for key {key:?}"))
    }

    pub fn values(&self) -> impl Iterator<Item = &ChannelSpec> {
        self.map.values()
    }
}

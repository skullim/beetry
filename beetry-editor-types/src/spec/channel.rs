use super::message::{MessageHashProvider, MessageTypeProvider};
use beetry_core::MessageHash;
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

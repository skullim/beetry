use crate::id::ChannelSpecId;
use anyhow::{Result, anyhow};
use derive_more::{Display, From};
use getset::{CopyGetters, MutGetters, Setters};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelData {
    pub spec_id: ChannelSpecId,
    pub config: ChannelConfig,
}

impl ChannelData {
    pub fn new(spec_id: ChannelSpecId, config: ChannelConfig) -> Self {
        Self { spec_id, config }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CopyGetters, Setters, MutGetters)]
pub struct ChannelConfig {
    #[getset(get_copy = "pub", set = "pub")]
    capacity: usize, // there might be channels with 0 capacity
    #[getset(get_copy = "pub", get_mut = "pub")]
    count: SenderReceiverCount,
    #[getset(get_copy = "pub", set = "pub")]
    kind: ChannelKind,
}

impl ChannelConfig {
    pub fn new(input: ChannelConfigInput) -> Self {
        Self {
            capacity: input.capacity,
            kind: input.kind,
            count: <_>::default(),
        }
    }
}

impl From<ChannelConfig> for ChannelConfigInput {
    fn from(value: ChannelConfig) -> Self {
        Self {
            capacity: value.capacity(),
            kind: value.kind(),
        }
    }
}

impl From<ChannelConfig> for ChannelConfigUpdate {
    fn from(value: ChannelConfig) -> Self {
        Self {
            capacity: value.capacity(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChannelConfigInput {
    pub capacity: usize,
    pub kind: ChannelKind,
}

impl ChannelConfigInput {
    pub fn new(capacity: usize, kind: ChannelKind) -> Self {
        Self { capacity, kind }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChannelConfigUpdate {
    pub capacity: usize,
}

#[derive(Debug, Display, From, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ChannelKind {
    Tokio(TokioChannelKind),
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokioChannelKind {
    Mpsc,
    Broadcast,
}

/// Represents the current state of connected senders and receivers
/// On channel creation there are no senders and receivers
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SenderReceiverCount {
    sender: usize,
    receiver: usize,
}

impl SenderReceiverCount {
    pub fn increase_sender_count(&mut self) {
        self.sender += 1;
    }

    pub fn decrease_sender_count(&mut self) -> Result<()> {
        self.sender = self
            .sender
            .checked_sub(1)
            .ok_or_else(|| anyhow!("cannot decrease sender count below 0"))?;
        Ok(())
    }

    pub fn increase_receiver_count(&mut self) {
        self.receiver += 1;
    }

    pub fn decrease_receiver_count(&mut self) -> Result<()> {
        self.receiver = self
            .receiver
            .checked_sub(1)
            .ok_or_else(|| anyhow!("cannot decrease receiver count below 0"))?;
        Ok(())
    }
}

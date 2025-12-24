use anyhow::{Result, bail};
use derive_more::{AddAssign, Display, From};
use getset::{CopyGetters, Getters, MutGetters, Setters};
use num_traits::One;
use serde::{Deserialize, Serialize};

use beetry_plugin_types::channel::ChannelSpec;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct ChannelSnapshot2 {
    spec: ChannelSpec,
    config: ChannelConfig,
}

impl ChannelSnapshot2 {
    pub fn new(spec: ChannelSpec, config: ChannelConfig) -> Self {
        Self { spec, config }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CopyGetters, Setters, MutGetters)]
pub struct ChannelConfig {
    #[getset(get_copy = "pub", set = "pub")]
    capacity: usize, // there might be channels with 0 capacity
    #[getset(get_copy = "pub", get_mut = "pub")]
    count: SenderReceiverCount,
    #[getset(get_copy = "pub")]
    kind: ChannelKind,
}

impl ChannelConfig {
    pub fn new(capacity: usize, kind: ChannelKind) -> Self {
        Self {
            capacity,
            kind,
            count: <_>::default(),
        }
    }

    pub fn set_kind(&mut self, kind: ChannelKind) -> Result<()> {
        // validate if kind is valid, e.g. if there are multiple receivers and one tries to change to mpsc
        todo!()
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Display,
    AddAssign,
)]
#[serde(transparent)]
pub struct ChannelId {
    id: u16,
}

impl ChannelId {
    pub fn new(id: u16) -> Self {
        Self { id }
    }

    pub fn next(&self) -> Self {
        ChannelId { id: self.id + 1 }
    }
}

impl One for ChannelId {
    fn one() -> Self {
        Self::new(1)
    }
}

impl std::ops::Mul for ChannelId {
    type Output = ChannelId;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            id: self.id * rhs.id,
        }
    }
}

#[derive(Debug, From, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ChannelKind {
    Tokio(TokioChannelKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
        match self.sender.checked_sub(1) {
            Some(count) => self.sender = count,
            None => {
                bail!("cannot decrease sender count below 0");
            }
        }
        Ok(())
    }

    pub fn increase_receiver_count(&mut self) {
        self.receiver += 1;
    }

    pub fn decrease_receiver_count(&mut self) -> Result<()> {
        match self.receiver.checked_sub(1) {
            Some(count) => self.receiver = count,
            None => {
                bail!("cannot decrease receiver count below 0");
            }
        }
        Ok(())
    }
}

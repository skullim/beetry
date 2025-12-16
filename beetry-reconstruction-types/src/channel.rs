use std::collections::HashMap;
use std::num::NonZeroUsize;

use anyhow::{Result, bail};
use bon::Builder;
use derive_more::{AddAssign, Display, From};
use getset::{CopyGetters, Getters, MutGetters};
use num_traits::One;
use serde::{Deserialize, Serialize};

use crate::tree::{ExportResult, ExportValidationError};
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CopyGetters, MutGetters)]
pub struct ChannelConfig {
    #[getset(get_copy = "pub", set = "pub")]
    capacity: usize, // there might be channels with 0 capacity
    #[getset(get_copy = "pub", get_mut = "pub")]
    count: SenderReceiverCount,
    #[getset(get_copy = "pub")]
    kind: ChannelImplKind2,
}

impl ChannelConfig {
    pub fn new(capacity: usize, kind: ChannelImplKind2) -> Self {
        Self {
            capacity,
            kind,
            count: <_>::default(),
        }
    }

    pub fn set_kind(&mut self, kind: ChannelImplKind2) {
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

//@todo: rename to ChannelKind
#[derive(Debug, From, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ChannelImplKind2 {
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

//@todo remove

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct ChannelSnapshot {
    spec: ChannelSpec,
    metadata: ChannelMetadata,
}

impl ChannelSnapshot {
    pub fn new(spec: ChannelSpec, metadata: ChannelMetadata) -> Self {
        Self { spec, metadata }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CopyGetters, Getters)]
pub struct ChannelMetadata {
    #[get_copy = "pub"]
    capacity: usize, // there might be channels with 0 capacity
    #[get_copy = "pub"]
    kind: ChannelKind,
    #[get = "pub"]
    impl_kind: ChannelImplKind,
}

impl ChannelMetadata {
    pub fn new(capacity: usize, kind: ChannelKind, impl_kind: ChannelImplKind) -> Self {
        Self {
            capacity,
            kind,
            impl_kind,
        }
    }
}

//@todo remove from here to below, validation should not be needed anymore
#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum ChannelKind {
    Internal,
    External,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChannelImplKind {
    Tokio(TokioChannelConfig),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokioChannelConfig {
    Mpsc(MpscConfig),
    Broadcast(BroadcastConfig),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CopyGetters)]
pub struct MpscConfig {
    #[get_copy = "pub"]
    n_senders: NonZeroUsize,
}

impl MpscConfig {
    pub fn new(n_senders: NonZeroUsize) -> Self {
        Self { n_senders }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize, CopyGetters)]
#[get_copy = "pub"]
pub struct BroadcastConfig {
    n_senders: NonZeroUsize,
    n_receivers: NonZeroUsize,
}

pub type ChannelSnapshotMap = HashMap<ChannelId, ChannelSnapshot>;
pub(super) struct ChannelValidator;
pub(super) type ChannelIdEndpointCountMap = HashMap<ChannelId, SenderReceiverCount>;

impl ChannelValidator {
    pub(super) fn validate(
        snapshot_map: &ChannelSnapshotMap,
        count_map: &ChannelIdEndpointCountMap,
    ) -> ExportResult<()> {
        for (id, count) in count_map {
            let snapshot = Self::validate_channel_presence(snapshot_map, *id)?;
            Self::validate_endpoint_count(snapshot, *id, count)?;
        }
        Ok(())
    }

    fn validate_channel_presence(
        snapshot_map: &ChannelSnapshotMap,
        id: ChannelId,
    ) -> ExportResult<&ChannelSnapshot> {
        snapshot_map
            .get(&id)
            .map_or(Err(ExportValidationError::ChannelNotFound(id)), |snap| {
                Ok(snap)
            })
    }

    const fn validate_endpoint_count(
        snapshot: &ChannelSnapshot,
        id: ChannelId,
        count: &SenderReceiverCount,
    ) -> ExportResult<()> {
        match &snapshot.metadata.impl_kind {
            ChannelImplKind::Tokio(tokio_config) => match tokio_config {
                TokioChannelConfig::Mpsc(mpsc_config) => {
                    let expected_senders = mpsc_config.n_senders.get();
                    if count.sender != expected_senders {
                        return Err(ExportValidationError::SenderCountMismatch(
                            id,
                            expected_senders,
                            count.sender,
                        ));
                    }
                }
                TokioChannelConfig::Broadcast(broadcast_config) => {
                    let expected_senders = broadcast_config.n_senders.get();
                    let expected_receivers = broadcast_config.n_receivers.get();

                    if count.sender != expected_senders {
                        return Err(ExportValidationError::SenderCountMismatch(
                            id,
                            expected_senders,
                            count.sender,
                        ));
                    }

                    if count.receiver != expected_receivers {
                        return Err(ExportValidationError::ReceiverCountMismatch(
                            id,
                            expected_receivers,
                            count.receiver,
                        ));
                    }
                }
            },
        }
        Ok(())
    }
}

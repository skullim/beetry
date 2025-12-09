use std::collections::HashMap;
use std::num::NonZeroUsize;

use anyhow::Result;
use bon::Builder;
use derive_more::{Display, From};
use getset::{CopyGetters, Getters, MutGetters};
use serde::{Deserialize, Serialize};

use crate::de::tree::{ExportResult, ExportValidationError};
use crate::ser::channel::ChannelSpec;

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

//@todo remove
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CopyGetters, MutGetters)]
pub struct ChannelConfig {
    #[getset(get_copy = "pub")]
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

    pub fn set_capacity(&mut self, capacity: usize) -> Result<()> {
        // might return err when setting capacity for oneshot channel (once this channel type is supported)
        self.capacity = capacity;
        Ok(())
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

//@todo remove
#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum ChannelKind {
    Internal,
    External,
}

//@todo remove
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChannelImplKind {
    Tokio(TokioChannelConfig),
}

//@todo remove
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokioChannelConfig {
    Mpsc(MpscConfig),
    Broadcast(BroadcastConfig),
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

    pub fn decrease_sender_count(&mut self) {
        self.sender -= 1;
    }

    pub fn increase_receiver_count(&mut self) {
        self.receiver += 1;
    }

    pub fn decrease_receiver_count(&mut self) {
        self.receiver -= 1;
    }
}

pub type ChannelIdToSnapshotMap = HashMap<ChannelId, ChannelSnapshot>;
pub(super) struct ChannelValidator;
pub(super) type ChannelIdEndpointCountMap = HashMap<ChannelId, SenderReceiverCount>;

impl ChannelValidator {
    pub(super) fn validate(
        snapshot_map: &ChannelIdToSnapshotMap,
        count_map: &ChannelIdEndpointCountMap,
    ) -> ExportResult<()> {
        for (id, count) in count_map {
            let snapshot = Self::validate_channel_presence(snapshot_map, *id)?;
            Self::validate_endpoint_count(snapshot, *id, count)?;
        }
        Ok(())
    }

    fn validate_channel_presence(
        snapshot_map: &ChannelIdToSnapshotMap,
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

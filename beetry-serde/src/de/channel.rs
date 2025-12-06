use std::collections::HashMap;
use std::num::NonZeroUsize;

use bon::Builder;
use derive_getters::Getters;
use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

use crate::de::tree::{ExportResult, ExportValidationError};
use crate::ser::channel::ChannelSpec;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
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
    metadata: ChannelParameters,
}

impl ChannelSnapshot2 {
    pub fn new(spec: ChannelSpec, metadata: ChannelParameters) -> Self {
        Self { spec, metadata }
    }
}

//@todo remove
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct ChannelMetadata {
    #[getter(copy)]
    capacity: usize, // there might be channels with 0 capacity
    #[getter(copy)]
    kind: ChannelKind,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct ChannelParameters {
    #[getter(copy)]
    capacity: usize, // there might be channels with 0 capacity
    count: SenderReceiverCount,
    kind: ChannelImplKind2,
}

impl ChannelParameters {
    pub fn new(capacity: usize, kind: ChannelImplKind2) -> Self {
        Self {
            capacity,
            count: SenderReceiverCount {
                sender: 1,
                receiver: 1,
            },
            kind,
        }
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
#[derive(Debug, From, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChannelImplKind2 {
    Tokio(TokioChannelKind),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokioChannelKind {
    Mpsc,
    Broadcast,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct MpscConfig {
    #[getter(copy)]
    n_senders: NonZeroUsize,
}

impl MpscConfig {
    pub fn new(n_senders: NonZeroUsize) -> Self {
        Self { n_senders }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize, Getters)]
pub struct BroadcastConfig {
    #[getter(copy)]
    n_senders: NonZeroUsize,
    #[getter(copy)]
    n_receivers: NonZeroUsize,
}

//@todo maybe default should be sender = 1, receiver = 1
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SenderReceiverCount {
    pub sender: usize,
    pub receiver: usize,
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

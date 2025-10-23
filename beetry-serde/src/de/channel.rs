use std::{collections::HashMap, num::NonZeroUsize};

use bon::Builder;
use derive_getters::Getters;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::{
    de::tree::{ExportResult, ExportValidationError},
    ser::channel::ChannelSpec,
};

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

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub enum ChannelKind {
    Internal,
    External,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChannelImplKind {
    Tokio(TokioChannelConfig),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokioChannelConfig {
    Mpsc(MpscConfig),
    Broadcast(BroadcastConfig),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct MpscConfig {
    #[getter(copy)]
    n_senders: NonZeroUsize,
}

impl MpscConfig {
    pub fn new(n_senders: NonZeroUsize) -> Self {
        Self { n_senders }
    }
}

#[derive(Debug, Clone, PartialEq, Builder, Serialize, Deserialize, Getters)]
pub struct BroadcastConfig {
    #[getter(copy)]
    n_senders: NonZeroUsize,
    #[getter(copy)]
    n_receivers: NonZeroUsize,
}

#[derive(Debug, Default)]
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
            let snapshot = Self::validate_channel_presence(snapshot_map, id)?;
            Self::validate_endpoint_count(snapshot, *id, count)?;
        }
        Ok(())
    }

    fn validate_channel_presence<'a>(
        snapshot_map: &'a ChannelIdToSnapshotMap,
        id: &ChannelId,
    ) -> ExportResult<&'a ChannelSnapshot> {
        match snapshot_map.get(id) {
            Some(e) => Ok(e),
            None => Err(ExportValidationError::ChannelNotFound(*id)),
        }
    }

    fn validate_endpoint_count(
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

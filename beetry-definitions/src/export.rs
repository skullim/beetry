use anyhow::{Result, anyhow};
use derive_getters::Getters;
use derive_more::Display;
use nonempty::NonEmpty;
use std::{
    collections::{BTreeSet, HashMap},
    num::NonZeroUsize,
};

use bon::{Builder, bon, builder};
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

use crate::{
    description::{ChannelDescription, LeafKind, MessageHash, NodeHash},
    parameter::SerializedParameters,
};

pub type ChannelIdToExportMap = HashMap<ChannelId, ChannelExport>;

#[derive(Debug, ThisError)]
pub enum ExportValidationError {
    #[error("channel {0:?} referenced but not defined")]
    ChannelNotFound(ChannelId),
    #[error("channel {0:?} has {1} senders configured but {2} actual senders")]
    SenderCountMismatch(ChannelId, usize, usize),
    #[error("channel {0:?} has {1} receivers configured but {2} actual receivers")]
    ReceiverCountMismatch(ChannelId, usize, usize),
}

pub type ExportResult<T> = std::result::Result<T, ExportValidationError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeExport {
    pub root: RootExport,
    pub channels: ChannelIdToExportMap,
}

#[bon]
impl TreeExport {
    #[builder]
    pub fn new(root: RootExport, channels: Option<ChannelIdToExportMap>) -> ExportResult<Self> {
        let export_map = channels.unwrap_or_default();
        let mut count_map = HashMap::new();
        Self::collect_channel_references(&root.child, &mut count_map);
        ChannelValidator::validate(&export_map, &count_map)?;

        Ok(TreeExport {
            root,
            channels: export_map,
        })
    }

    fn collect_channel_references(
        node: &NodeExport,
        map: &mut HashMap<ChannelId, SenderReceiverCount>,
    ) {
        match &node {
            NodeExport::Control(control) => {
                for child in &control.children {
                    Self::collect_channel_references(child, map);
                }
            }
            NodeExport::Leaf(leaf) => {
                for channel_id in &leaf.senders {
                    let count = map.entry(*channel_id).or_default();
                    count.sender += 1;
                }

                for channel_id in &leaf.receivers {
                    let count = map.entry(*channel_id).or_default();
                    count.receiver += 1;
                }
            }
        }
    }
}

struct ChannelValidator;

impl ChannelValidator {
    fn validate(
        export_map: &ChannelIdToExportMap,
        count_map: &ChannelIdEndpointCountMap,
    ) -> ExportResult<()> {
        for (id, count) in count_map {
            let export = Self::validate_channel_presence(export_map, id)?;
            Self::validate_endpoint_count(export, *id, count)?;
        }
        Ok(())
    }

    fn validate_channel_presence<'a>(
        export_map: &'a ChannelIdToExportMap,
        id: &ChannelId,
    ) -> ExportResult<&'a ChannelExport> {
        match export_map.get(id) {
            Some(e) => Ok(e),
            None => Err(ExportValidationError::ChannelNotFound(*id)),
        }
    }

    fn validate_endpoint_count(
        export: &ChannelExport,
        id: ChannelId,
        count: &SenderReceiverCount,
    ) -> ExportResult<()> {
        match &export.metadata.impl_kind {
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

type ChannelIdEndpointCountMap = HashMap<ChannelId, SenderReceiverCount>;

#[derive(Debug, Default)]
struct SenderReceiverCount {
    sender: usize,
    receiver: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
pub struct RootExport {
    child: NodeExport,
}

impl RootExport {
    pub fn new(child: NodeExport) -> Self {
        Self { child }
    }

    pub fn into_child(self) -> NodeExport {
        self.child
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeExport {
    Control(ControlExport),
    Leaf(LeafExport),
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
pub struct ControlExport {
    #[getter(copy)]
    kind: ControlKind,
    children: NonEmpty<Box<NodeExport>>,
}

impl ControlExport {
    pub fn new(
        kind: ControlKind,
        children: impl IntoIterator<Item = Box<NodeExport>>,
    ) -> Result<Self> {
        let children =
            NonEmpty::collect(children).ok_or(anyhow!("received empty children iterator"))?;
        Ok(Self { kind, children })
    }

    pub fn into_children_iter(self) -> impl IntoIterator<Item = Box<NodeExport>> {
        self.children.into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ControlKind {
    Sequence,
    Fallback,
    Parallel,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, Getters)]
pub struct LeafExport {
    #[builder(into)]
    name: String,
    #[getter(copy)]
    kind: LeafKind,
    #[getter(copy)]
    hash: NodeHash,
    #[builder(default, with = <_>::from_iter)]
    receivers: BTreeSet<ChannelId>,
    #[builder(default, with = <_>::from_iter)]
    senders: BTreeSet<ChannelId>,
    #[builder(default)]
    external_receivers_export: Vec<MessageHash>,
    #[builder(default)]
    parameters: SerializedParameters,
}

impl LeafExport {
    pub fn take_receivers(&mut self) -> impl IntoIterator<Item = ChannelId> {
        std::mem::take(&mut self.receivers)
    }

    pub fn take_external_receivers_export(
        &mut self,
    ) -> Option<impl IntoIterator<Item = MessageHash>> {
        if self.external_receivers_export.is_empty() {
            return None;
        }
        Some(std::mem::take(&mut self.external_receivers_export))
    }

    pub fn take_senders(&mut self) -> impl IntoIterator<Item = ChannelId> {
        std::mem::take(&mut self.senders)
    }

    pub fn take_parameters(&mut self) -> SerializedParameters {
        std::mem::take(&mut self.parameters)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct ChannelExport {
    desc: ChannelDescription,
    metadata: ChannelMetadata,
}

impl ChannelExport {
    pub fn new(desc: ChannelDescription, metadata: ChannelMetadata) -> Self {
        Self { desc, metadata }
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

#[cfg(test)]
mod tests {
    use type_hash::TypeHash;

    use super::*;
    use crate::description::{Message, NodeHash};
    use std::num::NonZeroUsize;

    #[derive(TypeHash)]
    struct Pose;

    impl Message for Pose {}

    fn create_test_leaf(
        name: &str,
        senders: impl Into<BTreeSet<ChannelId>>,
        receivers: impl Into<BTreeSet<ChannelId>>,
    ) -> NodeExport {
        NodeExport::Leaf(
            LeafExport::builder()
                .name(name)
                .kind(LeafKind::Action)
                .hash(NodeHash::new(0))
                .senders(senders.into())
                .receivers(receivers.into())
                .build(),
        )
    }

    fn create_test_sequence(children: Vec<NodeExport>) -> NodeExport {
        NodeExport::Control(ControlExport {
            kind: ControlKind::Sequence,
            children: NonEmpty::from_vec(children.into_iter().map(Box::new).collect()).unwrap(),
        })
    }

    fn create_mpsc_channel(id: ChannelId, n_senders: usize) -> (ChannelId, ChannelExport) {
        (
            id,
            ChannelExport {
                desc: ChannelDescription::new::<Pose>(),
                metadata: ChannelMetadata {
                    capacity: 10,
                    kind: ChannelKind::Internal,
                    impl_kind: ChannelImplKind::Tokio(TokioChannelConfig::Mpsc(MpscConfig {
                        n_senders: NonZeroUsize::new(n_senders).unwrap(),
                    })),
                },
            },
        )
    }

    fn create_broadcast_channel(
        id: ChannelId,
        n_senders: usize,
        n_receivers: usize,
    ) -> (ChannelId, ChannelExport) {
        (
            id,
            ChannelExport {
                desc: ChannelDescription::new::<Pose>(),
                metadata: ChannelMetadata {
                    capacity: 10,
                    kind: ChannelKind::Internal,
                    impl_kind: ChannelImplKind::Tokio(TokioChannelConfig::Broadcast(
                        BroadcastConfig {
                            n_senders: NonZeroUsize::new(n_senders).unwrap(),
                            n_receivers: NonZeroUsize::new(n_receivers).unwrap(),
                        },
                    )),
                },
            },
        )
    }

    #[test]
    fn test_valid_tree_export() {
        let channel1 = ChannelId::new(1);
        let channel2 = ChannelId::new(2);

        let root = RootExport {
            child: create_test_sequence(vec![
                create_test_leaf("sender", [channel1], []),
                create_test_leaf("receiver", [], [channel1]),
                create_test_leaf("broadcast_sender", [channel2], []),
                create_test_leaf("broadcast_receiver", [], [channel2]),
            ]),
        };

        let channels = HashMap::from([
            create_mpsc_channel(channel1, 1),
            create_broadcast_channel(channel2, 1, 1),
        ]);
        let result = TreeExport::builder().root(root).channels(channels).build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_channel_not_found() {
        let channel1 = ChannelId::new(1);
        let channel2 = ChannelId::new(2);

        let root = RootExport {
            child: create_test_leaf("sender", [channel1, channel2], []),
        };

        let channels = HashMap::from([create_mpsc_channel(channel1, 2)]);
        let result = TreeExport::builder().root(root).channels(channels).build();
        assert!((result.is_err()));
    }

    #[test]
    fn test_mpsc_sender_count_mismatch() {
        let channel1 = ChannelId::new(1);

        let root = RootExport {
            child: create_test_sequence(vec![
                create_test_leaf("sender1", [channel1], []),
                create_test_leaf("sender2", [channel1], []),
            ]),
        };

        // Configure for 1 sender but there are 2 senders
        let channels = HashMap::from([create_mpsc_channel(channel1, 1)]);
        let result = TreeExport::builder().root(root).channels(channels).build();
        assert!(matches!(
            result,
            Err(ExportValidationError::SenderCountMismatch(
                ChannelId { id: 1 },
                1,
                2
            ))
        ));
    }

    #[test]
    fn test_broadcast_sender_count_mismatch() {
        let channel1 = ChannelId::new(1);

        let root = RootExport {
            child: create_test_sequence(vec![
                create_test_leaf("sender1", [channel1], []),
                create_test_leaf("sender2", [channel1], []),
                create_test_leaf("receiver", [], [channel1]),
            ]),
        };

        // Configure for 1 sender but there are 2 senders
        let channels = HashMap::from([create_broadcast_channel(channel1, 1, 1)]);
        let result = TreeExport::builder().root(root).channels(channels).build();
        assert!(matches!(
            result,
            Err(ExportValidationError::SenderCountMismatch(
                ChannelId { id: 1 },
                1,
                2
            ))
        ));
    }

    #[test]
    fn test_broadcast_receiver_count_mismatch() {
        let channel1 = ChannelId::new(1);

        let root = RootExport {
            child: create_test_sequence(vec![
                create_test_leaf("sender", [channel1], []),
                create_test_leaf("receiver1", [], [channel1]),
                create_test_leaf("receiver2", [], [channel1]),
            ]),
        };

        // Configure for 1 receiver but there are 2 receivers
        let channels = HashMap::from([create_broadcast_channel(channel1, 1, 1)]);

        let result = TreeExport::builder().root(root).channels(channels).build();

        assert!(matches!(
            result,
            Err(ExportValidationError::ReceiverCountMismatch(
                ChannelId { id: 1 },
                1,
                2
            ))
        ));
    }

    #[test]
    fn test_empty_channels_map() {
        let root = RootExport {
            child: create_test_leaf("no_channels", [], []),
        };

        let result = TreeExport::builder().root(root).build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_nested_tree() {
        let channel1 = ChannelId::new(1);
        let channel2 = ChannelId::new(2);

        let root = RootExport {
            child: create_test_sequence(vec![
                create_test_sequence(vec![
                    create_test_leaf("sender1", [channel1], []),
                    create_test_leaf("sender2", [channel1], []),
                ]),
                create_test_sequence(vec![
                    create_test_leaf("receiver", [], [channel1]),
                    create_test_leaf("broadcast_node", [channel2], [channel2]),
                ]),
            ]),
        };

        let channels = HashMap::from([
            create_mpsc_channel(channel1, 2),
            create_broadcast_channel(channel2, 1, 1),
        ]);
        let result = TreeExport::builder().root(root).channels(channels).build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_channel_usage_same_node() {
        let channel1 = ChannelId::new(1);
        let channel2 = ChannelId::new(2);

        let root = RootExport {
            child: create_test_leaf(
                "multi_channel_node",
                [channel1, channel2],
                [channel1, channel2],
            ),
        };

        let channels = HashMap::from([
            create_broadcast_channel(channel1, 1, 1),
            create_broadcast_channel(channel2, 1, 1),
        ]);
        let result = TreeExport::builder().root(root).channels(channels).build();
        assert!(result.is_ok());
    }
}

use std::collections::HashMap;

use bon::bon;
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

use crate::de::{
    channel::{ChannelId, ChannelIdEndpointCountMap, ChannelIdToSnapshotMap, ChannelValidator},
    node::{NodeSnapshot, NodeSnapshotKind, RootSnapshot},
};

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
pub struct TreeSnapshot {
    pub root: RootSnapshot,
    pub channels: ChannelIdToSnapshotMap,
}

#[bon]
impl TreeSnapshot {
    #[builder]
    pub fn new(root: RootSnapshot, channels: Option<ChannelIdToSnapshotMap>) -> ExportResult<Self> {
        let snapshot_map = channels.unwrap_or_default();
        let mut count_map = HashMap::new();
        Self::collect_channel_references(&root.child, &mut count_map);
        ChannelValidator::validate(&snapshot_map, &count_map)?;

        Ok(TreeSnapshot {
            root,
            channels: snapshot_map,
        })
    }

    fn collect_channel_references(node: &NodeSnapshot, map: &mut ChannelIdEndpointCountMap) {
        match &node.kind {
            NodeSnapshotKind::Control(control) => {
                for child in control.children() {
                    Self::collect_channel_references(child, map);
                }
            }
            NodeSnapshotKind::Leaf(leaf) => {
                for channel_id in leaf.senders() {
                    let count = map.entry(*channel_id).or_default();
                    count.sender += 1;
                }

                for channel_id in leaf.receivers() {
                    let count = map.entry(*channel_id).or_default();
                    count.receiver += 1;
                }
            }
        }
    }
}

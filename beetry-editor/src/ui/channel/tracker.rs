use std::collections::{BTreeSet, HashMap};

use crate::{
    definitions::{NodeId, Point},
    ui::channel::ChannelElement,
};
use beetry_serde::de::channel::{ChannelExport, ChannelId};
use bon::Builder;
use dioxus_logger::tracing::debug;
use serde::{Deserialize, Serialize};

pub(crate) type ChannelIdToElementMap = HashMap<ChannelId, ChannelElement>;

#[derive(Debug, Default, Clone, Builder, Serialize, Deserialize)]
pub(crate) struct Tracker {
    channel_id: ChannelId,
    channels: ChannelIdToElementMap,
    senders: HashMap<NodeId, BTreeSet<ChannelId>>,
    receivers: HashMap<NodeId, BTreeSet<ChannelId>>,
}

impl Tracker {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn create_channel(&mut self, export: ChannelExport) {
        debug!(
            "creating new channel id: {:?} with message {:?}",
            self.channel_id,
            export.spec().as_str()
        );

        self.channels
            .insert(self.channel_id, ChannelElement::new(export));
        self.channel_id = self.channel_id.next();
    }

    pub(crate) fn connect_sender(&mut self, from: NodeId, id: ChannelId) {
        debug!("connecting sender from node: {from} to channel {id}");
        self.insert_sender(from, id);
    }

    pub(crate) fn connect_receiver(&mut self, from: NodeId, id: ChannelId) {
        debug!("connecting receiver from node: {from} to channel {id}");
        self.insert_receiver(from, id);
    }

    pub(crate) fn update_channel_pose(&mut self, id: ChannelId, pos: Point) {
        self.channels
            .entry(id)
            .and_modify(|element| element.pos = pos);
    }

    pub(crate) fn remove_node(&mut self, id: NodeId) {
        let mut references = vec![];
        if let Some(recv_references) = self.receivers.remove(&id) {
            references.extend(recv_references);
        }
        if let Some(send_references) = self.senders.remove(&id) {
            references.extend(send_references);
        }

        debug!("collected channel id references: {references:?} related to node id {id}");
        for reference in references {
            self.channels.remove(&reference);
            for set in self.receivers.values_mut() {
                set.remove(&reference);
            }
            for set in self.senders.values_mut() {
                set.remove(&reference);
            }
        }
    }

    fn insert_receiver(&mut self, to: NodeId, id: ChannelId) {
        self.receivers
            .entry(to)
            .and_modify(|receivers| {
                receivers.insert(id);
            })
            .or_insert_with(|| {
                let mut receivers = BTreeSet::new();
                receivers.insert(id);
                receivers
            });
    }

    fn insert_sender(&mut self, from: NodeId, id: ChannelId) {
        self.senders
            .entry(from)
            .and_modify(|senders| {
                senders.insert(id);
            })
            .or_insert_with(|| {
                let mut senders = BTreeSet::new();
                senders.insert(id);
                senders
            });
    }

    pub(crate) fn channels(&self) -> ChannelIdToElementMap {
        self.channels.clone()
    }

    pub(crate) fn channel(&self, id: ChannelId) -> Option<&ChannelElement> {
        self.channels.get(&id)
    }

    pub(crate) fn senders(&self) -> HashMap<NodeId, BTreeSet<ChannelId>> {
        self.senders.clone()
    }

    pub(crate) fn receivers(&self) -> HashMap<NodeId, BTreeSet<ChannelId>> {
        self.receivers.clone()
    }
}

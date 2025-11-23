use crate::domain::models::{ChannelId, EdgeId, ExternalReceivers, ExternalSenders, NodeEdge};

use super::models::{NodeId, NodeKind, NodePosition};
use beetry_core::MessageHash;
use beetry_serde::{
    de::parameter::Parameters,
    ser::node::{NodeName, NodeSpec},
};

use anyhow::Result;

pub trait NodeRepository {
    //@todo pass probably schema here
    // should repository deal with assigning node id's? Probably yes
    fn create_node(&mut self) -> Result<NodeId>;

    fn delete_node(&mut self, id: NodeId) -> Result<()>;

    fn kind(&self, id: NodeId) -> Option<NodeKind>;

    fn position(&self, id: NodeId) -> Option<NodePosition>;

    fn name(&self, id: NodeId) -> Option<&NodeName>;

    fn spec<T>(&self, id: NodeId) -> Option<&NodeSpec<T>>;

    fn nodes(&self) -> &[NodeId];
}

pub trait ParamRepository {
    fn insert(&mut self, id: NodeId, params: Parameters);

    fn params(&self, id: NodeId) -> &Parameters;
}

pub trait ExternalPortRepository {
    fn insert_sender(&mut self, id: NodeId, sender: MessageHash);

    fn senders(&self, id: NodeId) -> &ExternalSenders;

    fn insert_receiver(&mut self, id: NodeId, receiver: MessageHash);

    fn receivers(&self, id: NodeId) -> &ExternalReceivers;
}

pub trait EdgeRepository {
    fn create_edge(&mut self, edge: NodeEdge) -> Result<EdgeId>;

    fn delete_edge(&mut self, id: EdgeId) -> Result<()>;

    fn edges(&self) -> &[EdgeId];
}

pub trait ChannelRepository {
    fn create_channel(&mut self) -> Result<ChannelId>;

    fn delete_channel(&mut self, id: ChannelId) -> Result<()>;

    fn channels(&self) -> &[ChannelId];
}

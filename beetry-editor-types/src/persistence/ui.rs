use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    id::{ChannelId, NodeId, PortConnectionId},
    output::ui::{ChannelUiData, NodeUiData, PortConnectionUiData},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Store {
    pub nodes: Vec<NodeRecord>,
    pub channels: Vec<ChannelRecord>,
    pub port_connections: Vec<PortConnectionRecord>,
}

impl Store {
    pub fn new(
        nodes: impl IntoIterator<Item = NodeRecord>,
        channels: impl IntoIterator<Item = ChannelRecord>,
        port_connections: impl IntoIterator<Item = PortConnectionRecord>,
    ) -> Self {
        Self {
            nodes: nodes.into_iter().collect(),
            channels: channels.into_iter().collect(),
            port_connections: port_connections.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NodeRecord {
    pub id: NodeId,
    pub data: NodeUiData,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChannelRecord {
    pub id: ChannelId,
    pub data: ChannelUiData,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PortConnectionRecord {
    pub id: PortConnectionId,
    pub data: PortConnectionUiData,
}

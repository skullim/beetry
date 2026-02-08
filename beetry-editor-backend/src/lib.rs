mod domain;
mod id;

use anyhow::{Result, anyhow};
use beetry_core::MessageHash;
use beetry_editor_types::{
    spec::channel::ChannelSpec,
    spec::node::{NodeSpec, NodeSpecKey},
};
use domain::repository::{
    ChannelRepositoryFacade, EdgeRepository, NodeRepositoryFacade, UiRepositoryFacade,
};
use std::collections::HashMap;

pub use crate::domain::api;
pub type EditorService =
    domain::service::editor::EditorService<
        NodeRepositoryFacade,
        EdgeRepository,
        ChannelRepositoryFacade,
        UiRepositoryFacade,
    >;

#[derive(Debug, Clone)]
pub struct NodeSpecMap {
    map: HashMap<NodeSpecKey, NodeSpec>,
}

impl FromIterator<(NodeSpecKey, NodeSpec)> for NodeSpecMap {
    fn from_iter<T: IntoIterator<Item = (NodeSpecKey, NodeSpec)>>(iter: T) -> Self {
        Self {
            map: iter.into_iter().collect(),
        }
    }
}

impl NodeSpecMap {
    pub fn spec(&self, key: &NodeSpecKey) -> Result<&NodeSpec> {
        self.map
            .get(key)
            .ok_or_else(|| anyhow!("failed to obtain node spec for key {key:?}"))
    }

    pub fn values(&self) -> impl Iterator<Item = &NodeSpec> {
        self.map.values()
    }
}

#[derive(Debug, Clone)]
pub struct ChannelSpecMap {
    map: HashMap<MessageHash, ChannelSpec>,
}

impl FromIterator<(MessageHash, ChannelSpec)> for ChannelSpecMap {
    fn from_iter<T: IntoIterator<Item = (MessageHash, ChannelSpec)>>(iter: T) -> Self {
        Self {
            map: iter.into_iter().collect(),
        }
    }
}

impl ChannelSpecMap {
    pub fn spec(&self, key: &MessageHash) -> Result<&ChannelSpec> {
        self.map
            .get(key)
            .ok_or_else(|| anyhow!("failed to obtain channel spec for key {key:?}"))
    }

    pub fn values(&self) -> impl Iterator<Item = &ChannelSpec> {
        self.map.values()
    }
}

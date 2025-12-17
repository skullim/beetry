use anyhow::{Result, anyhow};
use beetry_reconstruction_types::channel::ChannelId;

use crate::domain::{
    models::{ChannelPosition, ChannelUiData, NodeId, NodeKind, NodePosition, NodeUiData},
    repository::UiRepositoryConcept,
    service::{channel::ChannelService, node::NodeService},
};

pub struct NodeUiServiceApi<'a, UR> {
    service: &'a NodeService,
    repo: &'a mut UR,
}

impl<'a, UR> NodeUiServiceApi<'a, UR>
where
    UR: UiRepositoryConcept<Id = NodeId, Data = NodeUiData>,
{
    fn create(&mut self, id: NodeId, data: NodeUiData) -> Result<()> {
        self.repo.create(id, data)
    }

    //@todo maybe should be pulled up?
    pub fn positions_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = &NodePosition> {
        self.service.positions_by_kind(self.repo, kind)
    }

    pub(super) fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        let data = self
            .repo
            .data_mut(id)
            .ok_or_else(|| anyhow!("unable to retrieve node {id} data"))?;
        data.position = position;
        Ok(())
    }
}

pub struct ChannelUiServiceApi<'a, UR> {
    service: &'a ChannelService,
    repo: &'a mut UR,
}

impl<'a, UR> ChannelUiServiceApi<'a, UR>
where
    UR: UiRepositoryConcept<Id = ChannelId, Data = ChannelUiData>,
{
    pub fn create(&mut self, id: ChannelId, data: ChannelUiData) -> Result<()> {
        self.repo.create(id, data)
    }

    pub(super) fn update_position(
        &mut self,
        id: ChannelId,
        position: ChannelPosition,
    ) -> Result<()> {
        let data = self
            .repo
            .data_mut(id)
            .ok_or_else(|| anyhow!("unable to retrieve channel {id} data"))?;
        data.position = position;
        Ok(())
    }
}

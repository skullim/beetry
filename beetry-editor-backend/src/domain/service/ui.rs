use std::collections::HashMap;

use anyhow::{Result, anyhow};
use beetry_editor_types::{
    id::{ChannelId, NodeId},
    output::ui::{ChannelUiData, NodeUiData, Point},
    spec::node::NodeKind,
};

use crate::domain::{
    repository::{
        UiRepositoryConcept, UiRepositoryFacadeConcept, UiRepositoryFacadeView,
        UiRepositoryFacadeViewMut,
    },
    service::node::NodeService,
};

pub struct UiBorrowApi<'a, URF>
where
    URF: UiRepositoryFacadeConcept,
{
    facade_view: UiRepositoryFacadeView<'a, URF>,
    service: &'a NodeService,
}

impl<'a, URF> UiBorrowApi<'a, URF>
where
    URF: UiRepositoryFacadeConcept,
{
    pub(super) fn new(
        facade_view: UiRepositoryFacadeView<'a, URF>,
        service: &'a NodeService,
    ) -> Self {
        Self {
            facade_view,
            service,
        }
    }

    pub fn node(&self) -> NodeUiBorrowApi<'_, URF::UiNodeRepo> {
        NodeUiBorrowApi {
            repo: self.facade_view.node,
            service: self.service,
        }
    }

    pub fn channel(&self) -> ChannelUiBorrowApi<'_, URF::UiChannelRepo> {
        ChannelUiBorrowApi {
            repo: self.facade_view.channel,
        }
    }
}

pub struct UiBorrowMutApi<'a, URF>
where
    URF: UiRepositoryFacadeConcept,
{
    facade_view: UiRepositoryFacadeViewMut<'a, URF>,
}

impl<'a, URF> UiBorrowMutApi<'a, URF>
where
    URF: UiRepositoryFacadeConcept,
{
    pub(super) fn new(facade_view: UiRepositoryFacadeViewMut<'a, URF>) -> Self {
        Self { facade_view }
    }

    pub fn node(&mut self) -> NodeUiBorrowMutApi<'_, URF::UiNodeRepo> {
        NodeUiBorrowMutApi {
            repo: self.facade_view.node,
        }
    }

    pub fn channel(&mut self) -> ChannelUiBorrowMutApi<'_, URF::UiChannelRepo> {
        ChannelUiBorrowMutApi {
            repo: self.facade_view.channel,
        }
    }
}

pub struct NodeUiBorrowApi<'a, UR> {
    service: &'a NodeService,
    repo: &'a UR,
}

impl<'a, UR> NodeUiBorrowApi<'a, UR>
where
    UR: UiRepositoryConcept<Id = NodeId, Data = NodeUiData>,
{
    pub fn data(&self, id: NodeId) -> Result<&NodeUiData> {
        self.repo
            .data(id)
            .ok_or_else(|| anyhow!("unable to retrieve node {id} data"))
    }

    pub fn position(&self, id: NodeId) -> Result<&Point> {
        Ok(&self.data(id)?.position)
    }

    pub fn positions(&self) -> impl Iterator<Item = &Point> {
        self.repo.data_iter().map(|data| &data.position)
    }

    pub fn sort_children(
        &self,
        ids: &mut [NodeId],
        sort_by: impl Fn(&NodeUiData, &NodeUiData) -> std::cmp::Ordering,
    ) -> Result<()> {
        let data_map = ids
            .iter()
            .map(|id| Ok((*id, self.data(*id)?)))
            .collect::<Result<HashMap<_, _>>>()?;
        ids.sort_by(|l, r| {
            // safe to access by index, data_map has to contain all the keys at this point
            let l_data = data_map[l];
            let r_data = data_map[r];
            sort_by(l_data, r_data)
        });
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&NodeId, &NodeUiData)> {
        self.repo.iter()
    }

    //@todo maybe should be pulled up, something like node cache service?
    pub fn positions_by_kind(&self, kind: NodeKind) -> impl Iterator<Item = (NodeId, &Point)> {
        self.service.positions_by_kind(self.repo, kind)
    }
}

pub struct NodeUiBorrowMutApi<'a, UR> {
    repo: &'a mut UR,
}

impl<'a, UR> NodeUiBorrowMutApi<'a, UR>
where
    UR: UiRepositoryConcept<Id = NodeId, Data = NodeUiData>,
{
    pub fn create(&mut self, id: NodeId, data: NodeUiData) -> Result<()> {
        self.repo.create(id, data)
    }

    pub fn remove(&mut self, id: NodeId) -> Option<NodeUiData> {
        self.repo.remove(id)
    }

    pub fn update_position(&mut self, id: NodeId, position: Point) -> Result<()> {
        let data = self
            .repo
            .data_mut(id)
            .ok_or_else(|| anyhow!("unable to retrieve node {id} data"))?;
        data.position = position;
        Ok(())
    }
}

pub struct ChannelUiBorrowApi<'a, UR> {
    repo: &'a UR,
}

impl<'a, UR> ChannelUiBorrowApi<'a, UR>
where
    UR: UiRepositoryConcept<Id = ChannelId, Data = ChannelUiData>,
{
    pub fn position(&self, id: ChannelId) -> Result<&Point> {
        Ok(&self
            .repo
            .data(id)
            .ok_or_else(|| anyhow!("failed to get channel {id} position"))?
            .position)
    }

    pub fn positions(&self) -> impl Iterator<Item = &Point> {
        self.repo.data_iter().map(|data| &data.position)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ChannelId, &ChannelUiData)> {
        self.repo.iter()
    }
}

pub struct ChannelUiBorrowMutApi<'a, UR> {
    repo: &'a mut UR,
}

impl<'a, UR> ChannelUiBorrowMutApi<'a, UR>
where
    UR: UiRepositoryConcept<Id = ChannelId, Data = ChannelUiData>,
{
    pub fn create(&mut self, id: ChannelId, data: ChannelUiData) -> Result<()> {
        self.repo.create(id, data)
    }

    pub fn update_position(&mut self, id: ChannelId, position: Point) -> Result<()> {
        let data = self
            .repo
            .data_mut(id)
            .ok_or_else(|| anyhow!("unable to retrieve channel {id} data"))?;
        data.position = position;
        Ok(())
    }
}

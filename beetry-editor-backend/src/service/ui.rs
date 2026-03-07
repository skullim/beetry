use crate::repository::{
    ChannelUiRepository, NodeUiRepository, PortConnectionRepository, PortConnectionUiRepository,
};
use anyhow::{Result, anyhow, bail};
use beetry_editor_types::{
    id::{ChannelId, NodeId, PortConnectionId},
    output::ui::{ChannelUiData, NodeUiData, Point, PortConnectionUiData},
};
use std::collections::HashMap;

pub struct NodeUiQueryView<'a> {
    repo: &'a NodeUiRepository,
}

pub trait NodeUiQuery {
    fn data(&self, id: NodeId) -> Result<&NodeUiData>;
    fn position(&self, id: NodeId) -> Result<&Point>;
    fn positions(&self) -> impl Iterator<Item = &Point>;
    fn iter(&self) -> impl Iterator<Item = (&NodeId, &NodeUiData)>;
}

impl<'a> NodeUiQueryView<'a> {
    pub(crate) fn new(repo: &'a NodeUiRepository) -> Self {
        Self { repo }
    }
}

impl NodeUiQuery for NodeUiQueryView<'_> {
    fn data(&self, id: NodeId) -> Result<&NodeUiData> {
        self.repo
            .data(id)
            .ok_or_else(|| anyhow!("unable to retrieve node {id} data"))
    }

    fn position(&self, id: NodeId) -> Result<&Point> {
        Ok(&self.data(id)?.position)
    }

    fn positions(&self) -> impl Iterator<Item = &Point> {
        self.repo.data_iter().map(|data| &data.position)
    }

    fn iter(&self) -> impl Iterator<Item = (&NodeId, &NodeUiData)> {
        self.repo.iter()
    }
}

pub struct NodeUiQueryProcessor<'a, Q> {
    query: &'a Q,
}

impl<'a, Q> NodeUiQueryProcessor<'a, Q>
where
    Q: NodeUiQuery,
{
    pub fn new(query: &'a Q) -> Self {
        Self { query }
    }

    pub fn sort_nodes(
        &self,
        ids: &mut [NodeId],
        sort_by: impl Fn(&NodeUiData, &NodeUiData) -> std::cmp::Ordering,
    ) -> Result<()> {
        let data_map = ids
            .iter()
            .map(|id| Ok((*id, self.query.data(*id)?)))
            .collect::<Result<HashMap<_, _>>>()?;
        ids.sort_by(|l, r| {
            let l_data = data_map[l];
            let r_data = data_map[r];
            sort_by(l_data, r_data)
        });
        Ok(())
    }

    pub fn map_to_positions(
        &self,
        id_iter: impl Iterator<Item = &'a NodeId>,
    ) -> impl Iterator<Item = Result<(&'a NodeId, &Point)>> {
        id_iter.map(|id| self.query.position(*id).map(|p| (id, p)))
    }
}

pub struct NodeUiViewMut<'a> {
    repo: &'a mut NodeUiRepository,
}

impl<'a> NodeUiViewMut<'a> {
    pub(crate) fn new(repo: &'a mut NodeUiRepository) -> Self {
        Self { repo }
    }

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

pub struct ChannelUiQueryView<'a> {
    repo: &'a ChannelUiRepository,
}

impl<'a> ChannelUiQueryView<'a> {
    pub(crate) fn new(repo: &'a ChannelUiRepository) -> Self {
        Self { repo }
    }
}

pub trait ChannelUiQuery {
    fn position(&self, id: ChannelId) -> Result<&Point>;
    fn positions(&self) -> impl Iterator<Item = &Point>;
    fn iter(&self) -> impl Iterator<Item = (&ChannelId, &ChannelUiData)>;
}

impl ChannelUiQuery for ChannelUiQueryView<'_> {
    fn position(&self, id: ChannelId) -> Result<&Point> {
        Ok(&self
            .repo
            .data(id)
            .ok_or_else(|| anyhow!("failed to get channel {id} position"))?
            .position)
    }
    fn positions(&self) -> impl Iterator<Item = &Point> {
        self.repo.data_iter().map(|data| &data.position)
    }
    fn iter(&self) -> impl Iterator<Item = (&ChannelId, &ChannelUiData)> {
        self.repo.iter()
    }
}

pub struct ChannelUiViewMut<'a> {
    repo: &'a mut ChannelUiRepository,
}

impl<'a> ChannelUiViewMut<'a> {
    pub(crate) fn new(repo: &'a mut ChannelUiRepository) -> Self {
        Self { repo }
    }

    pub fn create(&mut self, id: ChannelId, data: ChannelUiData) -> Result<()> {
        self.repo.create(id, data)
    }

    pub fn remove(&mut self, id: ChannelId) -> Option<ChannelUiData> {
        self.repo.remove(id)
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

pub struct PortConnectionUiStateViewMut<'a> {
    ui_repo: &'a mut PortConnectionUiRepository,
    port_conn_repo: &'a PortConnectionRepository,
}

impl<'a> PortConnectionUiStateViewMut<'a> {
    pub(crate) fn new(
        ui_repo: &'a mut PortConnectionUiRepository,
        port_conn_repo: &'a PortConnectionRepository,
    ) -> Self {
        Self {
            ui_repo,
            port_conn_repo,
        }
    }

    pub fn create(&mut self, id: PortConnectionId, data: PortConnectionUiData) -> Result<()> {
        if !self.port_conn_repo.conn_exists(id) {
            bail!("attempted to create ui connection state for non existing connection {id:?}")
        }
        self.ui_repo.create(id, data)
    }

    pub fn remove(&mut self, id: PortConnectionId) -> Option<PortConnectionUiData> {
        self.ui_repo.remove(id)
    }

    pub fn update(&mut self, id: PortConnectionId, new_data: PortConnectionUiData) -> Result<()> {
        let state = self
            .ui_repo
            .data_mut(id)
            .ok_or_else(|| anyhow!("unable to retrieve port connection {id:?} state"))?;
        *state = new_data;
        Ok(())
    }
}

pub trait PortConnectionUiQuery {
    fn data(&self, id: PortConnectionId) -> Result<&PortConnectionUiData>;
}

pub struct PortConnectionUiQueryView<'a> {
    repo: &'a PortConnectionUiRepository,
}

impl<'a> PortConnectionUiQueryView<'a> {
    pub fn new(repo: &'a PortConnectionUiRepository) -> Self {
        Self { repo }
    }
}

impl PortConnectionUiQuery for PortConnectionUiQueryView<'_> {
    fn data(&self, id: PortConnectionId) -> Result<&PortConnectionUiData> {
        self.repo
            .data(id)
            .ok_or_else(|| anyhow!("unable to retrieve ui port connection {id:?} data"))
    }
}

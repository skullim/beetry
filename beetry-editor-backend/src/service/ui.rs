use crate::repository::{PortConnectionRepositoryConcept, UiRepositoryConcept};
use anyhow::{Result, anyhow, bail};
use beetry_editor_types::{
    id::{ChannelId, NodeId, PortConnectionId},
    output::ui::{ChannelUiData, NodeUiData, Point, PortConnectionUiData},
};
use std::collections::HashMap;

pub struct NodeUiQueryView<'a, UR> {
    repo: &'a UR,
}

pub trait NodeUiQuery {
    fn data(&self, id: NodeId) -> Result<&NodeUiData>;
    fn position(&self, id: NodeId) -> Result<&Point>;
    fn positions(&self) -> impl Iterator<Item = &Point>;
    fn iter(&self) -> impl Iterator<Item = (&NodeId, &NodeUiData)>;
}

impl<'a, UR> NodeUiQueryView<'a, UR>
where
    UR: UiRepositoryConcept<Id = NodeId, Data = NodeUiData>,
{
    pub(crate) fn new(repo: &'a UR) -> Self {
        Self { repo }
    }
}

impl<UR> NodeUiQuery for NodeUiQueryView<'_, UR>
where
    UR: UiRepositoryConcept<Id = NodeId, Data = NodeUiData>,
{
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

pub struct NodeUiViewMut<'a, UR> {
    repo: &'a mut UR,
}

impl<'a, UR> NodeUiViewMut<'a, UR>
where
    UR: UiRepositoryConcept<Id = NodeId, Data = NodeUiData>,
{
    pub(crate) fn new(repo: &'a mut UR) -> Self {
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

pub struct ChannelUiQueryView<'a, UR> {
    repo: &'a UR,
}

impl<'a, UR> ChannelUiQueryView<'a, UR>
where
    UR: UiRepositoryConcept<Id = ChannelId, Data = ChannelUiData>,
{
    pub(crate) fn new(repo: &'a UR) -> Self {
        Self { repo }
    }
}

pub trait ChannelUiQuery {
    fn position(&self, id: ChannelId) -> Result<&Point>;
    fn positions(&self) -> impl Iterator<Item = &Point>;
    fn iter(&self) -> impl Iterator<Item = (&ChannelId, &ChannelUiData)>;
}

impl<UR> ChannelUiQuery for ChannelUiQueryView<'_, UR>
where
    UR: UiRepositoryConcept<Id = ChannelId, Data = ChannelUiData>,
{
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

pub struct ChannelUiViewMut<'a, UR> {
    repo: &'a mut UR,
}

impl<'a, UR> ChannelUiViewMut<'a, UR>
where
    UR: UiRepositoryConcept<Id = ChannelId, Data = ChannelUiData>,
{
    pub(crate) fn new(repo: &'a mut UR) -> Self {
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

pub struct PortConnectionUiStateViewMut<'a, UR, PC> {
    ui_repo: &'a mut UR,
    port_conn_repo: &'a PC,
}

impl<'a, UR, PC> PortConnectionUiStateViewMut<'a, UR, PC>
where
    UR: UiRepositoryConcept<Id = PortConnectionId, Data = PortConnectionUiData>,
    PC: PortConnectionRepositoryConcept,
{
    pub(crate) fn new(ui_repo: &'a mut UR, port_conn_repo: &'a PC) -> Self {
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

pub struct PortConnectionUiQueryView<'a, PUR> {
    repo: &'a PUR,
}

impl<'a, PUR> PortConnectionUiQueryView<'a, PUR> {
    pub fn new(repo: &'a PUR) -> Self {
        Self { repo }
    }
}

impl<'a, PUR> PortConnectionUiQuery for PortConnectionUiQueryView<'a, PUR>
where
    PUR: UiRepositoryConcept<Id = PortConnectionId, Data = PortConnectionUiData>,
{
    fn data(&self, id: PortConnectionId) -> Result<&PortConnectionUiData> {
        self.repo
            .data(id)
            .ok_or_else(|| anyhow!("unable to retrieve ui port connection {id:?} data"))
    }
}

use crate::domain::{
    models::{ChannelId, ChannelPosition, NodeId},
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        NodeRepositoryFacadeConcept, ParamRepositoryConcept,
    },
    service::node::NodeService,
};
use anyhow::{Result, bail};
use beetry_serde::ser::channel::ChannelSpec;

pub struct ChannelServiceView<'r, 'c, 'n, NRF, ER, CR, PR> {
    repo: &'r mut EditorRepository<NRF, ER, CR, PR>,
    channel: &'c mut ChannelService,
    node: &'n NodeService,
}

impl<'r, 'c, 'n, NRF, ER, CR, PR> ChannelServiceView<'r, 'c, 'n, NRF, ER, CR, PR>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelRepositoryConcept,
    PR: ParamRepositoryConcept,
{
    pub(crate) fn new(
        repo: &'r mut EditorRepository<NRF, ER, CR, PR>,
        channel: &'c mut ChannelService,
        node: &'n NodeService,
    ) -> Self {
        Self {
            repo,
            channel,
            node,
        }
    }

    pub fn create(&mut self, spec: ChannelSpec) -> Result<ChannelId> {
        self.channel.create(self.repo.channel_mut(), spec)
    }

    pub fn update_position(&mut self, id: ChannelId, position: ChannelPosition) -> Result<()> {
        self.channel.ensure_exists(self.repo.channel_mut(), id)?;
        self.channel
            .update_position(self.repo.channel_mut(), id, position)
    }

    pub fn connect_sender(&mut self, id: ChannelId, from: NodeId) -> Result<()> {
        self.node.ensure_exists(id, self.repo.node())?;
        self.channel
            .connect_sender(self.repo.channel_mut(), id, from);
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct ChannelService {
    id_assigner: ChannelIdAssigner,
}

impl ChannelService {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn on_node_removal<C>(repo: &mut C, id: NodeId) -> Result<()>
    where
        C: ChannelRepositoryConcept,
    {
        repo.on_node_removal(id)
    }

    fn create<C>(&mut self, repo: &mut C, spec: ChannelSpec) -> Result<ChannelId>
    where
        C: ChannelRepositoryConcept,
    {
        let id = self.id_assigner.next_id();
        repo.create(id, spec)?;
        Ok(id)
    }

    fn update_position<C>(
        &mut self,
        repo: &mut C,
        id: ChannelId,
        position: ChannelPosition,
    ) -> Result<()>
    where
        C: ChannelRepositoryConcept,
    {
        repo.update_position(id, position)
    }

    fn connect_sender<C>(&mut self, repo: &mut C, id: ChannelId, from: NodeId)
    where
        C: ChannelRepositoryConcept,
    {
        repo.insert_sender(id, from);
    }

    pub(crate) fn ensure_exists<C>(&self, repo: &mut C, id: ChannelId) -> Result<()>
    where
        C: ChannelRepositoryConcept,
    {
        if !repo.contains(id) {
            bail!("channel {id} does not exist")
        }
        Ok(())
    }
}

#[derive(Default)]
struct ChannelIdAssigner {
    id: ChannelId,
}

impl ChannelIdAssigner {
    fn next_id(&mut self) -> ChannelId {
        let id = self.id;
        self.id += 1;
        id
    }
}

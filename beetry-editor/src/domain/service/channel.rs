use crate::domain::{
    models::{ChannelId, ChannelPosition, NodeId},
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        NodeRepositoryFacadeConcept,
    },
    service::node::NodeService,
};
use anyhow::{Result, bail};
use beetry_serde::ser::channel::ChannelSpec;

pub struct ChannelServiceView<'r, 'c, 'n, NRF, ER, CR> {
    repo: &'r mut EditorRepository<NRF, ER, CR>,
    channel: &'c mut ChannelService,
    node: &'n NodeService,
}

impl<'r, 'c, 'n, NRF, ER, CR> ChannelServiceView<'r, 'c, 'n, NRF, ER, CR>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelRepositoryConcept,
{
    pub(crate) fn new(
        repo: &'r mut EditorRepository<NRF, ER, CR>,
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
        ChannelService::ensure_exists(self.repo.channel_mut(), id)?;
        ChannelService::update_position(self.repo.channel_mut(), id, position)
    }

    pub fn connect_sender(&mut self, id: ChannelId, from: NodeId) -> Result<()> {
        NodeService::ensure_exists(self.repo.node().view(), id)?;
        self.channel
            .connect_sender(self.repo.channel_mut(), id, from)
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

    pub(crate) fn on_node_removal(
        repo: &mut impl ChannelRepositoryConcept,
        id: NodeId,
    ) -> Result<()> {
        repo.on_node_removal(id)
    }

    fn create(
        &mut self,
        repo: &mut impl ChannelRepositoryConcept,
        spec: ChannelSpec,
    ) -> Result<ChannelId> {
        let id = self.id_assigner.next_id();
        repo.create(id, spec)?;
        Ok(id)
    }

    fn update_position(
        repo: &mut impl ChannelRepositoryConcept,
        id: ChannelId,
        position: ChannelPosition,
    ) -> Result<()> {
        repo.update_position(id, position)
    }

    fn connect_sender(
        &mut self,
        repo: &mut impl ChannelRepositoryConcept,
        id: ChannelId,
        from: NodeId,
    ) -> Result<()> {
        Self::ensure_exists(repo, id)?;
        repo.insert_sender(id, from);
        Ok(())
    }

    pub(crate) fn ensure_exists(
        repo: &mut impl ChannelRepositoryConcept,
        id: ChannelId,
    ) -> Result<()> {
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

use crate::domain::{
    models::{ChannelId, ChannelPosition, NodeId},
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        NodeRepositoryFacadeConcept,
    },
    service::node::NodeService,
};
use anyhow::{Result, anyhow, bail};
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
        ChannelService::ensure_exists(self.repo.channel(), id)?;
        ChannelService::update_position(self.repo.channel_mut(), id, position)
    }

    pub fn positions(&self) -> impl Iterator<Item = ChannelPosition> {
        ChannelService::positions(self.repo.channel())
    }

    pub fn spec(&self, id: ChannelId) -> Result<&ChannelSpec> {
        ChannelService::spec(self.repo.channel(), id)
    }

    pub fn channels(&self) -> impl Iterator<Item = ChannelId> {
        ChannelService::channels(self.repo.channel())
    }

    pub fn senders(&self, id: NodeId) -> impl Iterator<Item = ChannelId> {
        ChannelService::senders(self.repo.channel(), id)
    }

    pub fn receivers(&self, id: NodeId) -> impl Iterator<Item = ChannelId> {
        ChannelService::receivers(self.repo.channel(), id)
    }

    pub fn connect_sender(&mut self, id: ChannelId, from: NodeId) -> Result<()> {
        NodeService::ensure_exists(self.repo.node().view(), from)?;
        ChannelService::connect_sender(self.repo.channel_mut(), id, from)
    }

    pub fn connect_receiver(&mut self, id: ChannelId, to: NodeId) -> Result<()> {
        NodeService::ensure_exists(self.repo.node().view(), to)?;
        ChannelService::connect_receiver(self.repo.channel_mut(), id, to)
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

    fn positions(repo: &impl ChannelRepositoryConcept) -> impl Iterator<Item = ChannelPosition> {
        let ids = repo.channels();
        ids.flat_map(|id| repo.position(id).copied())
    }

    fn spec(repo: &impl ChannelRepositoryConcept, id: ChannelId) -> Result<&ChannelSpec> {
        repo.spec(id)
            .ok_or_else(|| anyhow!("no spec exists for channel {id}"))
    }

    fn connect_sender(
        repo: &mut impl ChannelRepositoryConcept,
        id: ChannelId,
        from: NodeId,
    ) -> Result<()> {
        //@todo: check w.r.t. spec and that node exists
        Self::ensure_exists(repo, id)?;
        //@todo increment sender count for mpsc setting
        repo.insert_sender(id, from);
        Ok(())
    }

    fn connect_receiver(
        repo: &mut impl ChannelRepositoryConcept,
        id: ChannelId,
        to: NodeId,
    ) -> Result<()> {
        //@todo: check w.r.t. spec and that node exists
        Self::ensure_exists(repo, id)?;
        //@todo increment receiver count for mpsc setting
        repo.insert_receiver(id, to);
        Ok(())
    }

    fn channels(repo: &impl ChannelRepositoryConcept) -> impl Iterator<Item = ChannelId> {
        repo.channels()
    }

    pub fn senders(
        repo: &impl ChannelRepositoryConcept,
        id: NodeId,
    ) -> impl Iterator<Item = ChannelId> {
        repo.senders(id)
    }

    pub fn receivers(
        repo: &impl ChannelRepositoryConcept,
        id: NodeId,
    ) -> impl Iterator<Item = ChannelId> {
        repo.receivers(id)
    }

    pub(crate) fn on_node_removal(
        repo: &mut impl ChannelRepositoryConcept,
        id: NodeId,
    ) -> Result<()> {
        repo.on_node_removal(id)
    }

    pub(crate) fn ensure_exists(repo: &impl ChannelRepositoryConcept, id: ChannelId) -> Result<()> {
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
        self.id = self.id.next();
        self.id
    }
}

use crate::domain::{
    models::{ChannelId, ChannelPosition, NodeChannelPortId, NodeChannelPortKind, NodeId},
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository, EditorRepositoryViewMut,
        NodeRepositoryFacadeConcept, NodeRepositoryFacadeView,
    },
    service::node::NodeService,
};
use anyhow::{Result, anyhow, bail};
use beetry_core::MessageHash;
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

    pub fn connect_sender(
        &mut self,
        id: ChannelId,
        from: NodeId,
        port_id: NodeChannelPortId,
    ) -> Result<()> {
        let EditorRepositoryViewMut { node, channel, .. } = self.repo.view_mut();
        ChannelService::connect_sender(channel, node, id, from, port_id)
    }

    pub fn connect_receiver(
        &mut self,
        id: ChannelId,
        to: NodeId,
        port_id: NodeChannelPortId,
    ) -> Result<()> {
        let EditorRepositoryViewMut { node, channel, .. } = self.repo.view_mut();
        ChannelService::connect_receiver(channel, node, id, to, port_id)
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
        channel_repo: &mut impl ChannelRepositoryConcept,
        node_repo: &impl NodeRepositoryFacadeConcept,
        id: ChannelId,
        from: NodeId,
        port_id: NodeChannelPortId,
    ) -> Result<()> {
        let node_view = node_repo.view();
        NodeService::ensure_exists(node_view, from)?;
        Self::ensure_exists(channel_repo, id)?;
        let channel_spec = Self::spec(channel_repo, id)?;
        Self::validate_channel_connection(node_view, *channel_spec.msg_hash(), id, from, port_id)?;
        //@todo increment sender count
        channel_repo.insert_sender(id, from);
        Ok(())
    }

    fn connect_receiver(
        channel_repo: &mut impl ChannelRepositoryConcept,
        node_repo: &impl NodeRepositoryFacadeConcept,
        id: ChannelId,
        to: NodeId,
        port_id: NodeChannelPortId,
    ) -> Result<()> {
        let node_view = node_repo.view();
        NodeService::ensure_exists(node_view, to)?;
        Self::ensure_exists(channel_repo, id)?;
        let channel_spec = Self::spec(channel_repo, id)?;
        Self::validate_channel_connection(node_view, *channel_spec.msg_hash(), id, to, port_id)?;
        //@todo increment receiver count
        channel_repo.insert_receiver(id, to);
        Ok(())
    }

    fn validate_channel_connection(
        node_view: NodeRepositoryFacadeView<'_, impl NodeRepositoryFacadeConcept>,
        channel_hash: MessageHash,
        id: ChannelId,
        from: NodeId,
        port_id: NodeChannelPortId,
    ) -> Result<()> {
        let port_hash = NodeService::port_hash(node_view.ports, from, port_id)?;
        let port_kind = NodeService::port_kind(node_view.ports, from, port_id)?;
        if port_hash != channel_hash || port_kind == NodeChannelPortKind::External {
            bail!(
                "attempted to connect mismatching channel {id} and node {from} port {port_id}, {port_kind:?}"
            );
        }
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

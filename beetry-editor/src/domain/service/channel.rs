use crate::domain::{
    models::{ChannelId, ChannelPosition, NodeChannelPortId, NodeId, NodePortConnection},
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository, EditorRepositoryViewMut,
        NodeRepositoryFacadeConcept, NodeRepositoryFacadeView,
    },
    service::node::NodeService,
};
use anyhow::{Result, anyhow, bail};
use beetry_serde::{
    de::channel::{ChannelImplKind2, ChannelParameters, TokioChannelKind},
    ser::channel::ChannelSpec,
};

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

    pub fn parameters(&self, id: ChannelId) -> Result<&ChannelParameters> {
        ChannelService::parameters(self.repo.channel(), id)
    }

    pub fn channels(&self) -> impl Iterator<Item = ChannelId> {
        ChannelService::channels(self.repo.channel())
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

    fn set_parameters(
        repo: &mut impl ChannelRepositoryConcept,
        id: ChannelId,
        metadata: ChannelParameters,
    ) -> Result<()> {
        repo.set_parameters(id, metadata)
    }

    fn parameters(
        repo: &impl ChannelRepositoryConcept,
        id: ChannelId,
    ) -> Result<&ChannelParameters> {
        repo.parameters(id)
            .ok_or_else(|| anyhow!("failed to obtain channel {id} parameters"))
    }

    fn parameters_mut(
        repo: &mut impl ChannelRepositoryConcept,
        id: ChannelId,
    ) -> Result<&mut ChannelParameters> {
        repo.parameters_mut(id)
            .ok_or_else(|| anyhow!("failed to obtain channel {id} parameters"))
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
        Self::validate_connection(node_view, channel_repo, id, from, port_id)?;
        Self::parameters_mut(channel_repo, id)?
            .count_mut()
            .increase_sender_count();
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
        Self::validate_connection(node_view, channel_repo, id, to, port_id)?;
        Self::parameters_mut(channel_repo, id)?
            .count_mut()
            .increase_receiver_count();
        Ok(())
    }

    //@todo implement API to remove node port <-> channel connection

    fn validate_connection(
        node_view: NodeRepositoryFacadeView<'_, impl NodeRepositoryFacadeConcept>,
        channel_repo: &impl ChannelRepositoryConcept,
        id: ChannelId,
        from: NodeId,
        port_id: NodeChannelPortId,
    ) -> Result<()> {
        let port_spec = NodeService::port_spec(node_view.ports, from, port_id)?;
        let channel_spec = Self::spec(channel_repo, id)?;
        if port_spec.msg_spec.hash() != channel_spec.msg_hash() {
            bail!("attempted to connect mismatched channel {id} and node {from} port {port_id}");
        }
        //@todo this check should be moved somewhere else, rationale: might hide different channels behind a feature gate
        let channel_params = Self::parameters(channel_repo, id)?;
        if let ChannelImplKind2::Tokio(TokioChannelKind::Mpsc) = channel_params.kind()
            && channel_params.count().receiver() == 1
        {
            bail!("attempted to create more than 1 receiver of mpsc channel");
        }

        let port_conn = NodeService::port_connection(node_view.ports, from, port_id)?;
        match port_conn {
            NodePortConnection::External => {
                bail!("attempted to connect to port {port_id} that is marked as external");
            }
            NodePortConnection::Internal(connected) => {
                if connected.contains(&id) {
                    bail!(
                        "connection between node {from} port {port_id} and channel {id} already exists"
                    );
                }
            }
        }

        Ok(())
    }

    fn channels(repo: &impl ChannelRepositoryConcept) -> impl Iterator<Item = ChannelId> {
        repo.channels()
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

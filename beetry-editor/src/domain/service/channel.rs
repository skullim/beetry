use std::collections::HashMap;

use crate::domain::{
    models::{ChannelId, ChannelPosition, ChannelSpecId, NodeId, NodePortConnection, NodePortId},
    ports::{
        ChannelData, ChannelDataRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        EditorRepositoryViewMut, NodeRepositoryFacadeConcept, NodeRepositoryFacadeView,
    },
    service::node::{self, NodeService},
};
use anyhow::{Result, anyhow, bail};
use beetry_serde::{
    de::channel::{ChannelConfig, ChannelImplKind2, TokioChannelKind},
    ser::channel::ChannelSpec,
};

pub struct ChannelServiceView<'r, 'c, NRF, ER, CR> {
    repo: &'r mut EditorRepository<NRF, ER, CR>,
    channel: &'c mut ChannelService,
}

impl<'r, 'c, NRF, ER, CR> ChannelServiceView<'r, 'c, NRF, ER, CR>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelDataRepositoryConcept,
{
    pub(crate) fn new(
        repo: &'r mut EditorRepository<NRF, ER, CR>,
        channel: &'c mut ChannelService,
    ) -> Self {
        Self { repo, channel }
    }

    pub fn create(&mut self, spec: ChannelSpec, data: ChannelData) -> Result<ChannelId> {
        self.channel.create(self.repo.channel_mut(), spec, data)
    }

    pub fn update_position(&mut self, id: ChannelId, position: ChannelPosition) -> Result<()> {
        ChannelService::ensure_exists(self.repo.channel(), id)?;
        ChannelService::update_position(self.repo.channel_mut(), id, position)
    }

    pub fn positions(&self) -> impl Iterator<Item = ChannelPosition> {
        ChannelService::positions(self.repo.channel())
    }

    pub fn parameters(&self, id: ChannelId) -> Result<&ChannelConfig> {
        ChannelService::config(self.repo.channel(), id)
    }

    pub fn channels(&self) -> impl Iterator<Item = ChannelId> {
        ChannelService::channels(self.repo.channel())
    }

    pub fn connect_sender(
        &mut self,
        id: ChannelId,
        from: NodeId,
        port_id: NodePortId,
    ) -> Result<()> {
        let EditorRepositoryViewMut { node, channel, .. } = self.repo.view_mut();
        ChannelService::connect_sender(channel, node, id, from, port_id)
    }

    pub fn connect_receiver(
        &mut self,
        id: ChannelId,
        to: NodeId,
        port_id: NodePortId,
    ) -> Result<()> {
        let EditorRepositoryViewMut { node, channel, .. } = self.repo.view_mut();
        ChannelService::connect_receiver(channel, node, id, to, port_id)
    }
}

#[derive(Default)]
pub(crate) struct ChannelService {
    spec_cache: HashMap<ChannelSpec, ChannelSpecId>,
    id_assigner: ChannelIdAssigner,
}

impl ChannelService {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn create(
        &mut self,
        repo: &mut impl ChannelDataRepositoryConcept,
        spec: ChannelSpec,
        data: ChannelData,
    ) -> Result<ChannelId> {
        let id = self.id_assigner.next_id();

        repo.create(id, data)?;
        Ok(id)
    }

    fn update_position(
        repo: &mut impl ChannelDataRepositoryConcept,
        id: ChannelId,
        position: ChannelPosition,
    ) -> Result<()> {
        Self::data_mut(repo, id)?.position = position;
        Ok(())
    }

    fn position(
        repo: &impl ChannelDataRepositoryConcept,
        id: ChannelId,
    ) -> Result<&ChannelPosition> {
        Ok(&Self::data(repo, id)?.position)
    }

    fn data(repo: &impl ChannelDataRepositoryConcept, id: ChannelId) -> Result<&ChannelData> {
        repo.data(id)
            .ok_or_else(|| anyhow!("failed to obtain data for channel {id}"))
    }

    fn data_mut(
        repo: &mut impl ChannelDataRepositoryConcept,
        id: ChannelId,
    ) -> Result<&mut ChannelData> {
        repo.data_mut(id)
            .ok_or_else(|| anyhow!("failed to obtain data for channel {id}"))
    }

    fn insert_config(
        repo: &mut impl ChannelDataRepositoryConcept,
        id: ChannelId,
        config: ChannelConfig,
    ) -> Result<()> {
        Self::data_mut(repo, id)?.config = config;
        Ok(())
    }

    fn config(repo: &impl ChannelDataRepositoryConcept, id: ChannelId) -> Result<&ChannelConfig> {
        Ok(&Self::data(repo, id)?.config)
    }

    fn config_mut(
        repo: &mut impl ChannelDataRepositoryConcept,
        id: ChannelId,
    ) -> Result<&mut ChannelConfig> {
        Ok(&mut Self::data_mut(repo, id)?.config)
    }

    fn connect_sender(
        channel_repo: &mut impl ChannelDataRepositoryConcept,
        node_repo: &impl NodeRepositoryFacadeConcept,
        id: ChannelId,
        from: NodeId,
        port_id: NodePortId,
    ) -> Result<()> {
        let node_view = node_repo.view();
        NodeService::ensure_exists(node_view, from)?;
        Self::ensure_exists(channel_repo, id)?;
        Self::validate_connection(node_view, channel_repo, id, from, port_id)?;
        Self::config_mut(channel_repo, id)?
            .count_mut()
            .increase_sender_count();
        Ok(())
    }

    fn connect_receiver(
        channel_repo: &mut impl ChannelDataRepositoryConcept,
        node_repo: &impl NodeRepositoryFacadeConcept,
        id: ChannelId,
        to: NodeId,
        port_id: NodePortId,
    ) -> Result<()> {
        let node_view = node_repo.view();
        NodeService::ensure_exists(node_view, to)?;
        Self::ensure_exists(channel_repo, id)?;
        Self::validate_connection(node_view, channel_repo, id, to, port_id)?;
        Self::config_mut(channel_repo, id)?
            .count_mut()
            .increase_receiver_count();
        Ok(())
    }

    //@todo implement API to remove node port <-> channel connection

    fn validate_connection(
        node_view: NodeRepositoryFacadeView<'_, impl NodeRepositoryFacadeConcept>,
        channel_repo: &impl ChannelDataRepositoryConcept,
        id: ChannelId,
        from: NodeId,
        port_id: NodePortId,
    ) -> Result<()> {
        let port_spec =
            node::SpecService::ports(node_view.specs, node_view.nodes, from)?.spec(port_id)?;
        let channel_spec = Self::spec(channel_repo, id)?;
        if port_spec.msg_spec.hash() != channel_spec.msg_hash() {
            bail!("attempted to connect mismatched channel {id} and node {from} port {port_id}");
        }
        //@todo this check should be moved somewhere else, rationale: might hide different channels behind a feature gate
        let channel_params = Self::config(channel_repo, id)?;
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

    fn channels(repo: &impl ChannelDataRepositoryConcept) -> impl Iterator<Item = ChannelId> {
        repo.channels()
    }

    pub(crate) fn ensure_exists(
        repo: &impl ChannelDataRepositoryConcept,
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
        self.id = self.id.next();
        self.id
    }
}

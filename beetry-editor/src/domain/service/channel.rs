use std::collections::HashMap;

use crate::domain::models::{NodePortKind, NodePortSpec};
use crate::domain::{
    models::{ChannelId, ChannelPosition, ChannelSpecId, NodeId},
    repository::{
        ChannelData, ChannelDataInput, ChannelDataRepositoryConcept,
        ChannelRepositoryFacadeConcept, ChannelRepositoryFacadeViewMut, SpecRepositoryConcept,
    },
};
use anyhow::{Result, anyhow, bail};
use beetry_plugin_types::channel::ChannelSpec;
use beetry_reconstruction_types::channel::{ChannelConfig, ChannelImplKind2, TokioChannelKind};

pub struct ConnectionContext<'a> {
    pub spec: &'a NodePortSpec,
    pub node: NodeId,
    pub channel: ChannelId,
}

pub struct ChannelServiceApi<'a, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    facade_view: ChannelRepositoryFacadeViewMut<'a, CRF>,
    channel: &'a mut ChannelService,
}

impl<'a, CRF> ChannelServiceApi<'a, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    pub(super) fn new(
        facade: ChannelRepositoryFacadeViewMut<'a, CRF>,
        channel: &'a mut ChannelService,
    ) -> Self {
        Self {
            facade_view: facade,
            channel,
        }
    }

    pub fn create(&mut self, spec: ChannelSpec, input: ChannelDataInput) -> Result<ChannelId> {
        self.channel
            .create(self.facade_view.spec, self.facade_view.data, spec, input)
    }

    pub fn remove(&mut self, id: ChannelId) -> Result<()> {
        ChannelService::remove(self.facade_view.data, id)
    }

    pub fn update_position(&mut self, id: ChannelId, position: ChannelPosition) -> Result<()> {
        ChannelService::update_position(self.facade_view.data, id, position)
    }

    pub fn positions(&self) -> impl Iterator<Item = &ChannelPosition> {
        ChannelService::positions(self.facade_view.data)
    }

    pub fn config(&self, id: ChannelId) -> Result<&ChannelConfig> {
        ChannelService::config(self.facade_view.data, id)
    }

    //@todo user should not specify the connection count, so restrict access to some subset
    pub fn config_mut(&mut self, id: ChannelId) -> Result<&mut ChannelConfig> {
        ChannelService::config_mut(self.facade_view.data, id)
    }

    pub fn channels(&self) -> impl Iterator<Item = &ChannelId> {
        ChannelService::channels(self.facade_view.data)
    }

    pub fn spec(&self, id: ChannelId) -> Result<&ChannelSpec> {
        ChannelService::spec(self.facade_view.spec, self.facade_view.data, id)
    }

    pub(super) fn connect(&mut self, context: ConnectionContext) -> Result<()> {
        ChannelService::connect(self.facade_view.spec, self.facade_view.data, context)
    }

    pub(super) fn disconnect(&mut self, id: ChannelId, kind: NodePortKind) -> Result<()> {
        ChannelService::disconnect(self.facade_view.data, id, kind)
    }
}

#[derive(Default)]
pub(super) struct ChannelService {
    spec_cache: HashMap<ChannelSpec, ChannelSpecId>,
}

impl ChannelService {
    pub(super) fn new() -> Self {
        Self::default()
    }

    fn spec<'a>(
        spec_repo: &'a impl SpecRepositoryConcept<Spec = ChannelSpec, SpecId = ChannelSpecId>,
        channel_repo: &impl ChannelDataRepositoryConcept,
        id: ChannelId,
    ) -> Result<&'a ChannelSpec> {
        let spec_id = Self::data(channel_repo, id)?.spec_id;
        spec_repo
            .spec(spec_id)
            .ok_or_else(|| anyhow!("no spec with id"))
    }

    fn create(
        &mut self,
        spec_repo: &mut impl SpecRepositoryConcept<Spec = ChannelSpec, SpecId = ChannelSpecId>,
        channel_repo: &mut impl ChannelDataRepositoryConcept,
        spec: ChannelSpec,
        input: ChannelDataInput,
    ) -> Result<ChannelId> {
        let spec_id = match self.spec_cache.get(&spec) {
            Some(id) => *id,
            None => {
                let spec_id = spec_repo.create(spec.clone())?;
                self.spec_cache.insert(spec, spec_id);
                spec_id
            }
        };

        channel_repo.create(ChannelData::new(spec_id, input))
    }

    //@todo also on_node_removal should remove connections to removed node
    fn remove(channel_repo: &mut impl ChannelDataRepositoryConcept, id: ChannelId) -> Result<()> {
        channel_repo.remove(id);
        Ok(())
    }

    fn update_position(
        repo: &mut impl ChannelDataRepositoryConcept,
        id: ChannelId,
        position: ChannelPosition,
    ) -> Result<()> {
        Self::data_mut(repo, id)?.position = position;
        Ok(())
    }

    fn positions(
        repo: &impl ChannelDataRepositoryConcept,
    ) -> impl Iterator<Item = &ChannelPosition> {
        repo.data_iter().map(|data| &data.position)
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

    fn connect(
        channel_spec_repo: &impl SpecRepositoryConcept<Spec = ChannelSpec, SpecId = ChannelSpecId>,
        channel_repo: &mut impl ChannelDataRepositoryConcept,
        context: ConnectionContext,
    ) -> Result<()> {
        Self::ensure_exists(channel_repo, context.channel)?;
        Self::validate_connection(
            channel_spec_repo,
            channel_repo,
            context.channel,
            context.node,
            context.spec,
        )?;
        let count_mut = Self::config_mut(channel_repo, context.channel)?.count_mut();
        match context.spec.kind {
            NodePortKind::Receiver => {
                count_mut.increase_receiver_count();
            }
            NodePortKind::Sender => {
                count_mut.increase_sender_count();
            }
        }
        Ok(())
    }

    fn disconnect(
        channel_repo: &mut impl ChannelDataRepositoryConcept,
        id: ChannelId,
        kind: NodePortKind,
    ) -> Result<()> {
        let count_mut = Self::config_mut(channel_repo, id)?.count_mut();
        match kind {
            NodePortKind::Receiver => count_mut.decrease_receiver_count(),
            NodePortKind::Sender => count_mut.decrease_sender_count(),
        }
    }

    fn validate_connection(
        channel_spec_repo: &impl SpecRepositoryConcept<Spec = ChannelSpec, SpecId = ChannelSpecId>,
        channel_repo: &impl ChannelDataRepositoryConcept,
        id: ChannelId,
        from: NodeId,
        port_spec: &NodePortSpec,
    ) -> Result<()> {
        let channel_spec = Self::spec(channel_spec_repo, channel_repo, id)?;
        if port_spec.msg_spec.hash() != channel_spec.msg_hash() {
            bail!(
                "attempted to connect mismatched channel {id} and node {from} of port name {}",
                port_spec.msg_spec.desc()
            );
        }
        //@todo this check should be moved somewhere else, rationale: might want to hide different channels behind a feature gate at some point
        let channel_params = Self::config(channel_repo, id)?;
        if let ChannelImplKind2::Tokio(TokioChannelKind::Mpsc) = channel_params.kind()
            && channel_params.count().receiver() == 1
        {
            bail!("attempted to create more than 1 receiver of mpsc channel");
        }
        Ok(())
    }

    fn channels(repo: &impl ChannelDataRepositoryConcept) -> impl Iterator<Item = &ChannelId> {
        repo.channels()
    }

    fn ensure_exists(repo: &impl ChannelDataRepositoryConcept, id: ChannelId) -> Result<()> {
        if !repo.contains(&id) {
            bail!("channel {id} does not exist")
        }
        Ok(())
    }
}

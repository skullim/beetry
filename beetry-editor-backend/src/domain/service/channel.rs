use std::collections::HashMap;

use crate::domain::repository::{
    ChannelRepositoryConcept, ChannelRepositoryFacadeConcept, ChannelRepositoryFacadeView,
    ChannelRepositoryFacadeViewMut, SpecRepositoryConcept,
};
use anyhow::{Result, anyhow, bail};
use beetry_editor_types::{
    id::{ChannelId, ChannelSpecId, NodeId},
    output::channel::{ChannelConfig, ChannelData, ChannelKind, TokioChannelKind},
    persistence::{ChannelRecord, ChannelSpecRecord},
    spec::channel::ChannelSpec,
    spec::node::{NodePortKind, NodePortSpec},
};
use tracing::warn;

pub struct ChannelView<'a, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    facade_view: ChannelRepositoryFacadeView<'a, CRF>,
}

impl<'a, CRF> ChannelView<'a, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    pub(super) fn new(facade_view: ChannelRepositoryFacadeView<'a, CRF>) -> Self {
        Self { facade_view }
    }
}

pub trait ChannelQueryView {
    fn data(&self, id: ChannelId) -> Result<&ChannelData>;
    fn config(&self, id: ChannelId) -> Result<&ChannelConfig>;
    fn channels(&self) -> impl Iterator<Item = &ChannelId>;
    fn spec_id(&self, id: ChannelId) -> Result<ChannelSpecId>;
    fn spec(&self, id: ChannelId) -> Result<&ChannelSpec>;
}

impl<CRF> ChannelQueryView for ChannelView<'_, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    fn data(&self, id: ChannelId) -> Result<&ChannelData> {
        ChannelService::data(self.facade_view.channel, id)
    }

    fn config(&self, id: ChannelId) -> Result<&ChannelConfig> {
        ChannelService::config(self.facade_view.channel, id)
    }

    fn channels(&self) -> impl Iterator<Item = &ChannelId> {
        ChannelService::channels(self.facade_view.channel)
    }

    fn spec_id(&self, id: ChannelId) -> Result<ChannelSpecId> {
        ChannelService::spec_id(self.facade_view.channel, id)
    }

    fn spec(&self, id: ChannelId) -> Result<&ChannelSpec> {
        ChannelService::spec(self.facade_view.spec, self.facade_view.channel, id)
    }
}

pub struct ChannelViewMut<'a, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    facade_view: ChannelRepositoryFacadeViewMut<'a, CRF>,
    channel: &'a mut ChannelService,
}

impl<'a, CRF> ChannelViewMut<'a, CRF>
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

    pub fn create(&mut self, spec: &ChannelSpec, config: ChannelConfig) -> Result<ChannelId> {
        self.channel.create(
            self.facade_view.spec,
            self.facade_view.channel,
            spec,
            config,
        )
    }

    pub fn remove(&mut self, id: ChannelId) -> Option<ChannelData> {
        ChannelService::remove(self.facade_view.channel, id)
    }

    pub fn set_capacity(&mut self, id: ChannelId, capacity: usize) -> Result<()> {
        ChannelService::set_capacity(self.facade_view.channel, id, capacity)
    }

    pub(super) fn connect(&mut self, context: &ConnectionContext) -> Result<()> {
        ChannelService::connect(self.facade_view.spec, self.facade_view.channel, context)
    }

    pub(super) fn disconnect(&mut self, id: ChannelId, kind: NodePortKind) -> Result<()> {
        ChannelService::disconnect(self.facade_view.channel, id, kind)
    }
}

pub struct ConnectionContext<'a> {
    pub spec: &'a NodePortSpec,
    pub node: NodeId,
    pub channel: ChannelId,
}

// This is only needed by import/export API which is user-facing API, therefore this is not public
pub(super) struct LoadChannelView<'a, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    facade_view: ChannelRepositoryFacadeViewMut<'a, CRF>,
    channel: &'a mut ChannelService,
}

impl<'a, CRF> LoadChannelView<'a, CRF>
where
    CRF: ChannelRepositoryFacadeConcept,
{
    pub(super) fn new(
        facade_view: ChannelRepositoryFacadeViewMut<'a, CRF>,
        channel: &'a mut ChannelService,
    ) -> Self {
        Self {
            facade_view,
            channel,
        }
    }

    pub(super) fn load_spec(&mut self, record: ChannelSpecRecord) -> Result<()> {
        self.channel.load_spec(self.facade_view.spec, record)
    }

    pub(super) fn load_channel(&mut self, record: ChannelRecord) -> Result<()> {
        self.facade_view.channel.load(record.id, record.data)
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
        channel_repo: &impl ChannelRepositoryConcept,
        id: ChannelId,
    ) -> Result<&'a ChannelSpec> {
        let spec_id = Self::data(channel_repo, id)?.spec_id;
        spec_repo
            .spec(spec_id)
            .ok_or_else(|| anyhow!("no spec with id {spec_id}"))
    }

    fn create(
        &mut self,
        spec_repo: &mut impl SpecRepositoryConcept<Spec = ChannelSpec, SpecId = ChannelSpecId>,
        channel_repo: &mut impl ChannelRepositoryConcept,
        spec: &ChannelSpec,
        config: ChannelConfig,
    ) -> Result<ChannelId> {
        let spec_id = if let Some(id) = self.spec_cache.get(spec) {
            *id
        } else {
            let spec_id = spec_repo.create(spec.clone())?;
            self.spec_cache.insert(spec.clone(), spec_id);
            spec_id
        };

        channel_repo.create(ChannelData::new(spec_id, config))
    }

    fn load_spec(
        &mut self,
        spec_repo: &mut impl SpecRepositoryConcept<Spec = ChannelSpec, SpecId = ChannelSpecId>,
        record: ChannelSpecRecord,
    ) -> Result<()> {
        if let Some(id) = self.spec_cache.get(&record.spec) {
            warn!("spec {id} was already loaded");
        } else {
            let ChannelSpecRecord { id, spec } = record;
            spec_repo.load(id, spec.clone())?;
            self.spec_cache.insert(spec, id);
        }
        Ok(())
    }

    fn remove(
        channel_repo: &mut impl ChannelRepositoryConcept,
        id: ChannelId,
    ) -> Option<ChannelData> {
        channel_repo.remove(id)
    }

    fn config(repo: &impl ChannelRepositoryConcept, id: ChannelId) -> Result<&ChannelConfig> {
        Ok(&Self::data(repo, id)?.config)
    }

    fn set_capacity(
        repo: &mut impl ChannelRepositoryConcept,
        id: ChannelId,
        capacity: usize,
    ) -> Result<()> {
        Self::config_mut(repo, id)?.set_capacity(capacity);
        Ok(())
    }

    fn config_mut(
        repo: &mut impl ChannelRepositoryConcept,
        id: ChannelId,
    ) -> Result<&mut ChannelConfig> {
        Ok(&mut Self::data_mut(repo, id)?.config)
    }

    fn spec_id(repo: &impl ChannelRepositoryConcept, id: ChannelId) -> Result<ChannelSpecId> {
        Ok(Self::data(repo, id)?.spec_id)
    }

    fn data(repo: &impl ChannelRepositoryConcept, id: ChannelId) -> Result<&ChannelData> {
        repo.data(id)
            .ok_or_else(|| anyhow!("failed to obtain data for channel {id}"))
    }

    fn data_mut(
        repo: &mut impl ChannelRepositoryConcept,
        id: ChannelId,
    ) -> Result<&mut ChannelData> {
        repo.data_mut(id)
            .ok_or_else(|| anyhow!("failed to obtain data for channel {id}"))
    }

    fn connect(
        channel_spec_repo: &impl SpecRepositoryConcept<Spec = ChannelSpec, SpecId = ChannelSpecId>,
        channel_repo: &mut impl ChannelRepositoryConcept,
        context: &ConnectionContext,
    ) -> Result<()> {
        Self::ensure_exists(channel_repo, context.channel)?;
        Self::validate_connection(channel_spec_repo, channel_repo, context)?;
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
        channel_repo: &mut impl ChannelRepositoryConcept,
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
        channel_repo: &impl ChannelRepositoryConcept,
        conn_ctx: &ConnectionContext,
    ) -> Result<()> {
        let channel_spec = Self::spec(channel_spec_repo, channel_repo, conn_ctx.channel)?;
        if conn_ctx.spec.msg_spec.hash() != channel_spec.msg_hash() {
            bail!(
                "failed to connect channel {} to node {} ({} port '{}'): type mismatch (channel type: '{}', port expects: '{}')",
                conn_ctx.channel,
                conn_ctx.node,
                conn_ctx.spec.kind.as_ref(),
                conn_ctx.spec.msg_spec.desc(),
                channel_spec.as_str(),
                conn_ctx.spec.msg_spec.desc(),
            );
        }
        //@todo this check should be moved somewhere else, rationale: might want to hide different channels behind a feature gate at some point
        let channel_params = Self::config(channel_repo, conn_ctx.channel)?;
        if let ChannelKind::Tokio(TokioChannelKind::Mpsc) = channel_params.kind()
            && conn_ctx.spec.kind == NodePortKind::Receiver
            && channel_params.count().receiver() == 1
        {
            bail!("attempted to create more than 1 receiver of mpsc channel");
        }
        Ok(())
    }

    fn channels(repo: &impl ChannelRepositoryConcept) -> impl Iterator<Item = &ChannelId> {
        repo.channels()
    }

    fn ensure_exists(repo: &impl ChannelRepositoryConcept, id: ChannelId) -> Result<()> {
        if !repo.contains(&id) {
            bail!("channel {id} does not exist")
        }
        Ok(())
    }
}

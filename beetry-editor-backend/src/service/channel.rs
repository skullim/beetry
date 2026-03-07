use std::collections::HashMap;

use crate::repository::{
    ChannelRepository, ChannelRepositoryFacadeView, ChannelRepositoryFacadeViewMut,
    ChannelSpecRepository,
};
use anyhow::{Result, anyhow, bail};
use beetry_editor_types::{
    id::{ChannelId, ChannelSpecId, NodeId},
    output::channel::{
        ChannelConfig, ChannelConfigInput, ChannelConfigUpdate, ChannelData, ChannelKind,
        TokioChannelKind,
    },
    persistence::{ChannelRecord, ChannelSpecRecord},
    spec::channel::ChannelSpec,
    spec::node::{NodePortKind, NodePortSpec},
};
use tracing::warn;

pub struct ChannelView<'a> {
    facade_view: ChannelRepositoryFacadeView<'a>,
}

impl<'a> ChannelView<'a> {
    pub(crate) fn new(facade_view: ChannelRepositoryFacadeView<'a>) -> Self {
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

impl ChannelQueryView for ChannelView<'_> {
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

pub struct ChannelViewMut<'a> {
    facade_view: ChannelRepositoryFacadeViewMut<'a>,
    channel: &'a mut ChannelService,
}

impl<'a> ChannelViewMut<'a> {
    pub(crate) fn new(
        facade: ChannelRepositoryFacadeViewMut<'a>,
        channel: &'a mut ChannelService,
    ) -> Self {
        Self {
            facade_view: facade,
            channel,
        }
    }

    pub fn create(&mut self, spec: &ChannelSpec, input: ChannelConfigInput) -> Result<ChannelId> {
        self.channel
            .create(self.facade_view.spec, self.facade_view.channel, spec, input)
    }

    pub fn remove(&mut self, id: ChannelId) -> Option<ChannelData> {
        ChannelService::remove(self.facade_view.channel, id)
    }

    pub fn update_config(&mut self, id: ChannelId, update: ChannelConfigUpdate) -> Result<()> {
        ChannelService::update_config(self.facade_view.channel, id, update)
    }

    pub(crate) fn validate_connection(&self, context: &ConnectionContext) -> Result<()> {
        ChannelService::validate_connection(
            self.facade_view.spec,
            self.facade_view.channel,
            context,
        )
    }

    pub(crate) fn on_connected(&mut self, context: &ConnectionContext) -> Result<()> {
        ChannelService::on_connected(self.facade_view.channel, context)
    }

    pub(crate) fn disconnect(&mut self, id: ChannelId, kind: NodePortKind) -> Result<()> {
        ChannelService::disconnect(self.facade_view.channel, id, kind)
    }
}

pub struct ConnectionContext<'a> {
    pub spec: &'a NodePortSpec,
    pub node: NodeId,
    pub channel: ChannelId,
}

// This is only needed by import/export API which is user-facing API, therefore this is not public
pub(crate) struct LoadChannelView<'a> {
    facade_view: ChannelRepositoryFacadeViewMut<'a>,
    channel: &'a mut ChannelService,
}

impl<'a> LoadChannelView<'a> {
    pub(crate) fn new(
        facade_view: ChannelRepositoryFacadeViewMut<'a>,
        channel: &'a mut ChannelService,
    ) -> Self {
        Self {
            facade_view,
            channel,
        }
    }

    pub(crate) fn load_spec(&mut self, record: ChannelSpecRecord) -> Result<()> {
        self.channel.load_spec(self.facade_view.spec, record)
    }

    pub(crate) fn load_channel(&mut self, record: ChannelRecord) -> Result<()> {
        self.facade_view.channel.load(record.id, record.data)
    }
}

#[derive(Default)]
pub(crate) struct ChannelService {
    spec_cache: HashMap<ChannelSpec, ChannelSpecId>,
}

impl ChannelService {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn spec<'a>(
        spec_repo: &'a ChannelSpecRepository,
        channel_repo: &ChannelRepository,
        id: ChannelId,
    ) -> Result<&'a ChannelSpec> {
        let spec_id = Self::data(channel_repo, id)?.spec_id;
        spec_repo
            .spec(spec_id)
            .ok_or_else(|| anyhow!("no spec with id {spec_id}"))
    }

    fn create(
        &mut self,
        spec_repo: &mut ChannelSpecRepository,
        channel_repo: &mut ChannelRepository,
        spec: &ChannelSpec,
        input: ChannelConfigInput,
    ) -> Result<ChannelId> {
        let spec_id = if let Some(id) = self.spec_cache.get(spec) {
            *id
        } else {
            let spec_id = spec_repo.create(spec.clone())?;
            self.spec_cache.insert(spec.clone(), spec_id);
            spec_id
        };

        channel_repo.create(ChannelData::new(spec_id, ChannelConfig::new(input)))
    }

    fn load_spec(
        &mut self,
        spec_repo: &mut ChannelSpecRepository,
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

    fn remove(channel_repo: &mut ChannelRepository, id: ChannelId) -> Option<ChannelData> {
        channel_repo.remove(id)
    }

    fn config(repo: &ChannelRepository, id: ChannelId) -> Result<&ChannelConfig> {
        Ok(&Self::data(repo, id)?.config)
    }

    fn update_config(
        repo: &mut ChannelRepository,
        id: ChannelId,
        update: ChannelConfigUpdate,
    ) -> Result<()> {
        Self::config_mut(repo, id)?.set_capacity(update.capacity);
        Ok(())
    }

    fn config_mut(repo: &mut ChannelRepository, id: ChannelId) -> Result<&mut ChannelConfig> {
        Ok(&mut Self::data_mut(repo, id)?.config)
    }

    fn spec_id(repo: &ChannelRepository, id: ChannelId) -> Result<ChannelSpecId> {
        Ok(Self::data(repo, id)?.spec_id)
    }

    fn data(repo: &ChannelRepository, id: ChannelId) -> Result<&ChannelData> {
        repo.data(id)
            .ok_or_else(|| anyhow!("failed to obtain data for channel {id}"))
    }

    fn data_mut(repo: &mut ChannelRepository, id: ChannelId) -> Result<&mut ChannelData> {
        repo.data_mut(id)
            .ok_or_else(|| anyhow!("failed to obtain data for channel {id}"))
    }

    fn on_connected(
        channel_repo: &mut ChannelRepository,
        context: &ConnectionContext,
    ) -> Result<()> {
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
        channel_repo: &mut ChannelRepository,
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
        channel_spec_repo: &ChannelSpecRepository,
        channel_repo: &ChannelRepository,
        conn_ctx: &ConnectionContext,
    ) -> Result<()> {
        Self::ensure_exists(channel_repo, conn_ctx.channel)?;
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
        let channel_params = Self::config(channel_repo, conn_ctx.channel)?;
        if let ChannelKind::Tokio(TokioChannelKind::Mpsc) = channel_params.kind()
            && conn_ctx.spec.kind == NodePortKind::Receiver
            && channel_params.count().receiver() == 1
        {
            bail!("attempted to create more than 1 receiver of mpsc channel");
        }
        Ok(())
    }

    fn channels(repo: &ChannelRepository) -> impl Iterator<Item = &ChannelId> {
        repo.channels()
    }

    fn ensure_exists(repo: &ChannelRepository, id: ChannelId) -> Result<()> {
        if !repo.contains(&id) {
            bail!("channel {id} does not exist")
        }
        Ok(())
    }
}

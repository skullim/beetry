use std::collections::HashMap;

use crate::domain::service::node::{PortStateServiceApi, SpecServiceApi, TrackerServiceApi};
use crate::domain::{
    models::{ChannelId, ChannelPosition, ChannelSpecId, NodeId, NodePortConnection, NodePortId},
    ports::{
        ChannelData, ChannelDataInput, ChannelDataRepositoryConcept,
        ChannelRepositoryFacadeConcept, ChannelRepositoryFacadeViewMut,
        NodeRepositoryFacadeConcept, SpecRepositoryConcept,
    },
};
use anyhow::{Result, anyhow, bail};
use beetry_serde::{
    de::channel::{ChannelConfig, ChannelImplKind2, TokioChannelKind},
    ser::channel::ChannelSpec,
};

pub(super) struct ExternalDeps<'a, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    tracker: TrackerServiceApi<'a, NRF::NodeRepo>,
    spec: SpecServiceApi<'a, NRF::SpecRepo, NRF::NodeRepo>,
    port_state: PortStateServiceApi<'a, NRF::PortStateRepo>,
}

impl<'a, NRF> ExternalDeps<'a, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    pub(super) fn new(
        tracker: TrackerServiceApi<'a, NRF::NodeRepo>,
        spec: SpecServiceApi<'a, NRF::SpecRepo, NRF::NodeRepo>,
        port_state: PortStateServiceApi<'a, NRF::PortStateRepo>,
    ) -> Self {
        Self {
            tracker,
            spec,
            port_state,
        }
    }
}

pub struct ChannelServiceApi<'a, CRF, NRF>
where
    CRF: ChannelRepositoryFacadeConcept,
    NRF: NodeRepositoryFacadeConcept,
{
    facade_view: ChannelRepositoryFacadeViewMut<'a, CRF>,
    channel: &'a mut ChannelService,
    deps: ExternalDeps<'a, NRF>,
}

impl<'a, CRF, NRF> ChannelServiceApi<'a, CRF, NRF>
where
    CRF: ChannelRepositoryFacadeConcept,
    NRF: NodeRepositoryFacadeConcept,
{
    pub(crate) fn new(
        facade: ChannelRepositoryFacadeViewMut<'a, CRF>,
        channel: &'a mut ChannelService,
        deps: ExternalDeps<'a, NRF>,
    ) -> Self {
        Self {
            facade_view: facade,
            channel,
            deps,
        }
    }

    pub fn create(&mut self, spec: ChannelSpec, input: ChannelDataInput) -> Result<ChannelId> {
        self.channel
            .create(self.facade_view.spec, self.facade_view.data, spec, input)
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

    //@todo user should not specify the connection count, so restrict access
    pub fn config_mut(&mut self, id: ChannelId) -> Result<&mut ChannelConfig> {
        ChannelService::config_mut(self.facade_view.data, id)
    }

    pub fn channels(&self) -> impl Iterator<Item = &ChannelId> {
        ChannelService::channels(self.facade_view.data)
    }

    pub fn connect_sender(
        &mut self,
        id: ChannelId,
        from: NodeId,
        port_id: NodePortId,
    ) -> Result<()> {
        ChannelService::connect_sender::<NRF>(
            self.facade_view.spec,
            self.facade_view.data,
            &self.deps,
            id,
            from,
            port_id,
        )
    }

    pub fn connect_receiver(
        &mut self,
        id: ChannelId,
        to: NodeId,
        port_id: NodePortId,
    ) -> Result<()> {
        ChannelService::connect_receiver::<NRF>(
            self.facade_view.spec,
            self.facade_view.data,
            &self.deps,
            id,
            to,
            port_id,
        )
    }

    pub fn spec(&self, id: ChannelId) -> Result<&ChannelSpec> {
        ChannelService::spec(self.facade_view.spec, self.facade_view.data, id)
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

    pub fn positions(
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

    fn connect_sender<NRF>(
        channel_spec_repo: &impl SpecRepositoryConcept<Spec = ChannelSpec, SpecId = ChannelSpecId>,
        channel_repo: &mut impl ChannelDataRepositoryConcept,
        deps: &ExternalDeps<'_, NRF>,
        id: ChannelId,
        from: NodeId,
        port_id: NodePortId,
    ) -> Result<()>
    where
        NRF: NodeRepositoryFacadeConcept,
    {
        deps.tracker.ensure_exists(from)?;
        Self::ensure_exists(channel_repo, id)?;
        Self::validate_connection::<NRF>(channel_spec_repo, channel_repo, deps, id, from, port_id)?;
        Self::config_mut(channel_repo, id)?
            .count_mut()
            .increase_sender_count();
        Ok(())
    }

    fn connect_receiver<NRF>(
        channel_spec_repo: &impl SpecRepositoryConcept<Spec = ChannelSpec, SpecId = ChannelSpecId>,
        channel_repo: &mut impl ChannelDataRepositoryConcept,
        deps: &ExternalDeps<'_, NRF>,
        id: ChannelId,
        to: NodeId,
        port_id: NodePortId,
    ) -> Result<()>
    where
        NRF: NodeRepositoryFacadeConcept,
    {
        deps.tracker.ensure_exists(to)?;
        Self::ensure_exists(channel_repo, id)?;
        Self::validate_connection::<NRF>(channel_spec_repo, channel_repo, deps, id, to, port_id)?;
        Self::config_mut(channel_repo, id)?
            .count_mut()
            .increase_receiver_count();
        Ok(())
    }

    //@todo implement API to remove node port <-> channel connection
    //@todo also on_node_removal should remove connections to removed node

    fn validate_connection<NRF>(
        channel_spec_repo: &impl SpecRepositoryConcept<Spec = ChannelSpec, SpecId = ChannelSpecId>,
        channel_repo: &impl ChannelDataRepositoryConcept,
        deps: &ExternalDeps<'_, NRF>,
        id: ChannelId,
        from: NodeId,
        port_id: NodePortId,
    ) -> Result<()>
    where
        NRF: NodeRepositoryFacadeConcept,
    {
        let channel_spec = Self::spec(channel_spec_repo, channel_repo, id)?;
        let port_spec = deps.spec.ports(from)?.spec(port_id)?;
        if port_spec.msg_spec.hash() != channel_spec.msg_hash() {
            bail!("attempted to connect mismatched channel {id} and node {from} port {port_id}");
        }
        //@todo this check should be moved somewhere else, rationale: might want to hide different channels behind a feature gate at some point
        let channel_params = Self::config(channel_repo, id)?;
        if let ChannelImplKind2::Tokio(TokioChannelKind::Mpsc) = channel_params.kind()
            && channel_params.count().receiver() == 1
        {
            bail!("attempted to create more than 1 receiver of mpsc channel");
        }
        let port_conn = deps.port_state.state(from, port_id)?;
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

    fn channels(repo: &impl ChannelDataRepositoryConcept) -> impl Iterator<Item = &ChannelId> {
        repo.channels()
    }

    pub(crate) fn ensure_exists(
        repo: &impl ChannelDataRepositoryConcept,
        id: ChannelId,
    ) -> Result<()> {
        if !repo.contains(&id) {
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

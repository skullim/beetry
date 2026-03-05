use crate::{
    repository::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, NodeRepositoryFacadeConcept,
        NodeRepositoryFacadeView, NodeRepositoryFacadeViewMut,
    },
    service::{
        channel::ChannelService,
        edge::{EdgeService, OnNodeRemovalServiceApi},
    },
};

use super::{
    NodeLifecycleView, NodeService, ParameterValueQueryView, PortConnectionQuery,
    PortConnectionQueryView, PortConnectionViewMut, PortSpecQuery, PortSpecQueryView,
    PortStateQueryView, PortStateViewMut, SpecView, TrackerView,
};

pub struct NodeView<'a, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    facade_view: NodeRepositoryFacadeView<'a, NRF>,
    node_service: &'a NodeService,
}
impl<'a, NRF> NodeView<'a, NRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    pub(crate) fn new(
        facade_view: NodeRepositoryFacadeView<'a, NRF>,
        node_service: &'a NodeService,
    ) -> Self {
        Self {
            facade_view,
            node_service,
        }
    }

    pub fn spec(&self) -> SpecView<'_, NRF::SpecRepo, NRF::NodeRepo> {
        SpecView {
            spec_repo: self.facade_view.specs,
            node_repo: self.facade_view.nodes,
        }
    }

    pub fn tracker(&self) -> TrackerView<'_, NRF::NodeRepo> {
        TrackerView {
            service: self.node_service,
            repo: self.facade_view.nodes,
        }
    }

    pub fn port_state(&self) -> PortStateQueryView<'_, NRF::PortStateRepo> {
        PortStateQueryView {
            repo: self.facade_view.ports,
        }
    }

    pub fn parameter(&self) -> ParameterValueQueryView<'_, NRF::ParamValuesRepo> {
        ParameterValueQueryView {
            repo: self.facade_view.parameters,
        }
    }

    pub fn port_connection_query(&self) -> impl PortConnectionQuery {
        PortConnectionQueryView::new(self.facade_view.port_connections)
    }

    pub fn port_spec_query(&self) -> impl PortSpecQuery {
        PortSpecQueryView::new(self.facade_view.nodes, self.facade_view.specs)
    }
}
/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct NodeViewMut<'a, NRF, ER, CRF>
where
    NRF: NodeRepositoryFacadeConcept,
{
    facade_view: NodeRepositoryFacadeViewMut<'a, NRF>,
    node_service: &'a mut NodeService,
    edge_repo: &'a mut ER,
    edge_service: &'a mut EdgeService,
    channel_facade: &'a mut CRF,
    channel_service: &'a mut ChannelService,
}

impl<'a, NRF, ER, CRF> NodeViewMut<'a, NRF, ER, CRF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    pub(crate) fn new(
        facade_view: NodeRepositoryFacadeViewMut<'a, NRF>,
        node_service: &'a mut NodeService,
        edge_repo: &'a mut ER,
        edge_service: &'a mut EdgeService,
        channel_facade: &'a mut CRF,
        channel_service: &'a mut ChannelService,
    ) -> Self {
        Self {
            facade_view,
            node_service,
            edge_repo,
            edge_service,
            channel_facade,
            channel_service,
        }
    }

    pub(crate) fn lifecycle(&'a mut self) -> NodeLifecycleView<'a, NRF, CRF, ER> {
        NodeLifecycleView {
            node_service: self.node_service,
            channel_facade: self.channel_facade,
            channel_service: self.channel_service,
            node_facade_view: &mut self.facade_view,
            edge_removal_service_api: OnNodeRemovalServiceApi::new(
                self.edge_service,
                self.edge_repo,
            ),
        }
    }

    pub fn port_state(
        &'a mut self,
    ) -> PortStateViewMut<
        'a,
        NRF::NodeRepo,
        NRF::SpecRepo,
        NRF::PortConnectionRepo,
        NRF::PortStateRepo,
        CRF,
    > {
        PortStateViewMut {
            node_repo: self.facade_view.nodes,
            spec_repo: self.facade_view.specs,
            port_conn_repo: self.facade_view.port_connections,
            port_state_repo: self.facade_view.ports,
            channel_facade: self.channel_facade,
            channel_service: self.channel_service,
        }
    }

    pub fn port_connection(
        &'a mut self,
    ) -> PortConnectionViewMut<
        'a,
        NRF::NodeRepo,
        NRF::SpecRepo,
        NRF::PortStateRepo,
        NRF::PortConnectionRepo,
        CRF,
    > {
        PortConnectionViewMut {
            node_repo: self.facade_view.nodes,
            spec_repo: self.facade_view.specs,
            port_conn_repo: self.facade_view.port_connections,
            port_state_repo: self.facade_view.ports,
            channel_facade: self.channel_facade,
            channel_service: self.channel_service,
        }
    }
}

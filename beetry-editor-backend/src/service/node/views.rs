use super::{
    NodeService, ParameterValueQueryView, PortConnectionQuery, PortConnectionQueryView,
    PortSpecQuery, PortSpecQueryView, PortStateQueryView, SpecView, TrackerView,
};
use crate::repository::NodeRepositoryFacadeView;

pub struct NodeView<'a> {
    facade_view: NodeRepositoryFacadeView<'a>,
    node_service: &'a NodeService,
}
impl<'a> NodeView<'a> {
    pub(crate) fn new(
        facade_view: NodeRepositoryFacadeView<'a>,
        node_service: &'a NodeService,
    ) -> Self {
        Self {
            facade_view,
            node_service,
        }
    }

    #[must_use]
    pub fn spec(&self) -> SpecView<'_> {
        SpecView {
            spec_repo: self.facade_view.spec,
            node_repo: self.facade_view.node,
        }
    }

    #[must_use]
    pub fn tracker(&self) -> TrackerView<'_> {
        TrackerView {
            service: self.node_service,
            repo: self.facade_view.node,
        }
    }

    #[must_use]
    pub fn port_state(&self) -> PortStateQueryView<'_> {
        PortStateQueryView {
            repo: self.facade_view.port_state,
        }
    }

    #[must_use]
    pub fn parameter(&self) -> ParameterValueQueryView<'_> {
        ParameterValueQueryView {
            repo: self.facade_view.parameter,
        }
    }

    #[must_use]
    pub fn port_connection_query(&self) -> impl PortConnectionQuery {
        PortConnectionQueryView::new(self.facade_view.port_connection)
    }

    #[must_use]
    pub fn port_spec_query(&self) -> impl PortSpecQuery {
        PortSpecQueryView::new(self.facade_view.node, self.facade_view.spec)
    }
}

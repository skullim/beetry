use anyhow::{Result, anyhow, bail};
use beetry_editor_types::{
    id::{NodeId, NodePortId, PortConnectionId},
    output::node::PortState,
    spec::node::{NodePortKind, PortsSpec},
};

use super::{SpecByNodeIdQuery, SpecByNodeIdQueryView, SpecBySpecIdQueryView, SpecView};
use crate::{
    repository::{
        ChannelRepository, ChannelRepositoryFacadeViewMut, ChannelSpecRepository, NodeRepository,
        NodeSpecRepository, PortConnectionRepository, PortStateRepository,
    },
    service::channel::{ChannelService, ChannelViewMut, ConnectionContext},
};

pub trait PortStateQuery {
    fn state(&self, node_id: NodeId, port_id: NodePortId) -> Result<&PortState>;
}

pub struct PortStateQueryView<'a> {
    pub(crate) repo: &'a PortStateRepository,
}

impl PortStateQuery for PortStateQueryView<'_> {
    fn state(&self, node_id: NodeId, port_id: NodePortId) -> Result<&PortState> {
        self.repo.state(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) state")
        })
    }
}

pub struct PortStateViewMut<'a> {
    pub(crate) node_repo: &'a NodeRepository,
    pub(crate) spec_repo: &'a NodeSpecRepository,
    pub(crate) port_conn_repo: &'a mut PortConnectionRepository,
    pub(crate) port_state_repo: &'a mut PortStateRepository,
    pub(crate) channel_repo: &'a mut ChannelRepository,
    pub(crate) channel_spec_repo: &'a mut ChannelSpecRepository,
    pub(crate) channel_service: &'a mut ChannelService,
}

impl PortStateViewMut<'_> {
    pub fn set_state(
        &mut self,
        node_id: NodeId,
        port_id: NodePortId,
        state: PortState,
    ) -> Result<()> {
        if state.is_external() {
            let mut port_conn_view = PortConnectionViewMut {
                node_repo: self.node_repo,
                spec_repo: self.spec_repo,
                port_conn_repo: self.port_conn_repo,
                port_state_repo: self.port_state_repo,
                channel_repo: self.channel_repo,
                channel_spec_repo: self.channel_spec_repo,
                channel_service: self.channel_service,
            };
            port_conn_view.disconnect_port(node_id, port_id)?;
        }
        self.port_state_repo.insert(node_id, port_id, state);
        Ok(())
    }
}

pub trait PortSpecQuery {
    fn port_order(&self, kind: NodePortKind, node_id: NodeId, port_id: NodePortId)
    -> Result<usize>;
}

pub struct PortSpecQueryView<'a> {
    node_repo: &'a NodeRepository,
    spec_repo: &'a NodeSpecRepository,
}

impl<'a> PortSpecQueryView<'a> {
    pub(crate) fn new(node_repo: &'a NodeRepository, spec_repo: &'a NodeSpecRepository) -> Self {
        Self {
            node_repo,
            spec_repo,
        }
    }
}

impl PortSpecQuery for PortSpecQueryView<'_> {
    fn port_order(
        &self,
        kind: NodePortKind,
        node_id: NodeId,
        port_id: NodePortId,
    ) -> Result<usize> {
        let spec_query =
            SpecByNodeIdQueryView::new(SpecBySpecIdQueryView::new(self.spec_repo), self.node_repo);
        let ports = spec_query.ports(node_id)?;
        match kind {
            NodePortKind::Sender => ports
                .sender_ids()
                .position(|id| *id == port_id)
                .ok_or_else(|| anyhow!("sender port {port_id} not found in node {node_id}")),
            NodePortKind::Receiver => ports
                .receiver_ids()
                .position(|id| *id == port_id)
                .ok_or_else(|| anyhow!("receiver port {port_id} not found in node {node_id}")),
        }
    }
}

pub trait PortConnectionQuery {
    fn connection_exists(&self, conn: PortConnectionId) -> bool;
    fn is_port_connected(&self, node: NodeId, port: NodePortId) -> bool;
    fn port_connections(
        &self,
        node: NodeId,
        port: NodePortId,
    ) -> impl Iterator<Item = PortConnectionId>;

    fn all_connections(&self) -> impl Iterator<Item = PortConnectionId>;
}

pub struct PortConnectionQueryView<'a> {
    repo: &'a PortConnectionRepository,
}

impl<'a> PortConnectionQueryView<'a> {
    pub(crate) fn new(repo: &'a PortConnectionRepository) -> Self {
        Self { repo }
    }
}

impl PortConnectionQuery for PortConnectionQueryView<'_> {
    fn connection_exists(&self, conn: PortConnectionId) -> bool {
        self.repo.conn_exists(conn)
    }

    fn is_port_connected(&self, node: NodeId, port: NodePortId) -> bool {
        self.repo.is_port_connected(node, port)
    }

    fn port_connections(
        &self,
        node: NodeId,
        port: NodePortId,
    ) -> impl Iterator<Item = PortConnectionId> {
        self.repo
            .node_conns(node)
            .filter(move |c| *c.0 == port)
            .map(move |c| PortConnectionId::new(node, *c.0, *c.1))
    }

    fn all_connections(&self) -> impl Iterator<Item = PortConnectionId> {
        self.repo.iter().flat_map(|(node_id, conns)| {
            conns.map(move |(port_id, channel_id)| {
                PortConnectionId::new(*node_id, *port_id, *channel_id)
            })
        })
    }
}

pub struct PortConnectionViewMut<'a> {
    pub(crate) node_repo: &'a NodeRepository,
    pub(crate) spec_repo: &'a NodeSpecRepository,
    pub(crate) port_state_repo: &'a PortStateRepository,
    pub(crate) port_conn_repo: &'a mut PortConnectionRepository,
    pub(crate) channel_repo: &'a mut ChannelRepository,
    pub(crate) channel_spec_repo: &'a mut ChannelSpecRepository,
    pub(crate) channel_service: &'a mut ChannelService,
}

impl PortConnectionViewMut<'_> {
    pub fn connect_port(&mut self, id: PortConnectionId) -> Result<()> {
        let query = PortStateQueryView {
            repo: self.port_state_repo,
        };
        if query
            .state(id.node_id, id.port_id)
            .is_ok_and(PortState::is_external)
        {
            bail!("attempted to connect port that is marked as external");
        }
        let mut coordinator = self.coordinator(id.node_id)?;
        coordinator.validate_connection(id)?;
        coordinator.on_connected(id)?;

        self.port_conn_repo.insert(id);
        Ok(())
    }

    pub fn disconnect(&mut self, id: PortConnectionId) -> Result<()> {
        self.port_conn_repo.remove(id);
        self.coordinator(id.node_id)?.on_disconnected(id)
    }

    pub fn disconnect_port(&mut self, node: NodeId, port: NodePortId) -> Result<()> {
        if let Some(channel_ids) = self.port_conn_repo.remove_port_conns(node, port) {
            for channel_id in channel_ids {
                self.disconnect(PortConnectionId::new(node, port, channel_id))?;
            }
        }
        Ok(())
    }

    pub fn disconnect_all_ports(&mut self, id: NodeId) -> Result<()> {
        if let Some(items) = self.port_conn_repo.remove_node_conns(id) {
            for (port_id, channel_id) in items {
                self.disconnect(PortConnectionId::new(id, port_id, channel_id))?;
            }
        }
        Ok(())
    }

    fn coordinator(&mut self, id: NodeId) -> Result<PortChannelCoordinator<'_>> {
        let spec = SpecView::spec_by_node_id(self.spec_repo, self.node_repo, id)?;
        let ports_spec = spec
            .ports()
            .as_ref()
            .ok_or_else(|| anyhow!("expected port specification for node {id}"))?;

        Ok(PortChannelCoordinator::new(
            ports_spec,
            ChannelViewMut::new(
                ChannelRepositoryFacadeViewMut {
                    spec: self.channel_spec_repo,
                    channel: self.channel_repo,
                },
                self.channel_service,
            ),
        ))
    }
}

pub struct PortChannelCoordinator<'a> {
    ports_spec: &'a PortsSpec,
    channel_service_view: ChannelViewMut<'a>,
}

impl<'a> PortChannelCoordinator<'a> {
    pub(crate) fn new(ports_spec: &'a PortsSpec, channel_service_view: ChannelViewMut<'a>) -> Self {
        Self {
            ports_spec,
            channel_service_view,
        }
    }
}

impl PortChannelCoordinator<'_> {
    fn validate_connection(&self, conn_id: PortConnectionId) -> Result<()> {
        self.channel_service_view
            .validate_connection(&Self::conn_ctx(self.ports_spec, conn_id)?)
    }

    fn on_connected(&mut self, conn_id: PortConnectionId) -> Result<()> {
        self.channel_service_view
            .on_connected(&Self::conn_ctx(self.ports_spec, conn_id)?)
    }

    fn on_disconnected(&mut self, conn_id: PortConnectionId) -> Result<()> {
        let spec = self.ports_spec.spec(conn_id.port_id)?;
        self.channel_service_view
            .disconnect(conn_id.channel_id, spec.kind)?;
        Ok(())
    }

    fn conn_ctx(
        ports_spec: &PortsSpec,
        conn_id: PortConnectionId,
    ) -> Result<ConnectionContext<'_>> {
        let spec = ports_spec.spec(conn_id.port_id)?;
        Ok(ConnectionContext {
            channel: conn_id.channel_id,
            node: conn_id.node_id,
            spec,
        })
    }
}

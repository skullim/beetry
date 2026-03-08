use crate::{
    repository::{ChannelRepository, ChannelSpecRepository, NodeRepositoryFacadeViewMut},
    service::{channel::ChannelService, edge},
};
use anyhow::Result;
use beetry_editor_types::persistence::PortsStateMap;
use beetry_editor_types::{
    id::{NodeId, NodePortId, NodeSpecId, PortConnectionId},
    output::node::{PortSource, PortState},
    persistence::ParameterValues,
    spec::node::NodeSpec,
};

use super::{NodeService, PortConnectionViewMut, PortStateViewMut};

pub struct LoadNodeView<'s, 'r> {
    node_service: &'s mut NodeService,
    node_facade_view: &'s mut NodeRepositoryFacadeViewMut<'r>,
}

impl<'s, 'r> LoadNodeView<'s, 'r> {
    pub(crate) fn new(
        node_service: &'s mut NodeService,
        node_facade_view: &'s mut NodeRepositoryFacadeViewMut<'r>,
    ) -> Self {
        Self {
            node_service,
            node_facade_view,
        }
    }

    pub(crate) fn load_node(
        &mut self,
        id: NodeId,
        spec_id: NodeSpecId,
        param_value: Option<ParameterValues>,
        ports_state: Option<PortsStateMap>,
    ) -> Result<()> {
        self.node_service.load_node(
            self.node_facade_view.spec,
            self.node_facade_view.node,
            id,
            spec_id,
        )?;
        if let Some(map) = ports_state {
            for (port_id, state) in map {
                self.load_port_state(id, port_id, state)?;
            }
        }

        if let Some(value) = param_value {
            self.load_parameters(id, value);
        }
        Ok(())
    }

    fn load_parameters(&mut self, id: NodeId, value: ParameterValues) {
        self.node_facade_view.parameter.create(id, value.params);
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "node existence check will make it fallible"
    )]
    fn load_port_state(&mut self, node: NodeId, port: NodePortId, state: PortState) -> Result<()> {
        self.node_facade_view.port_state.insert(node, port, state);
        Ok(())
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "node existence check will make it fallible"
    )]
    pub(crate) fn load_port_connections(
        &mut self,
        conns: impl IntoIterator<Item = PortConnectionId>,
    ) -> Result<()> {
        for conn in conns {
            self.node_facade_view.port_connection.insert(conn);
        }
        Ok(())
    }

    pub(crate) fn load_spec(&mut self, id: NodeSpecId, spec: NodeSpec) -> Result<()> {
        self.node_service
            .load_spec(self.node_facade_view.spec, id, spec)
    }
}

pub struct NodeLifecycleView<'a> {
    pub(crate) node_service: &'a mut NodeService,
    pub(crate) channel_service: &'a mut ChannelService,
    pub(crate) node_facade_view: NodeRepositoryFacadeViewMut<'a>,
    pub(crate) channel_repo: &'a mut ChannelRepository,
    pub(crate) channel_spec_repo: &'a mut ChannelSpecRepository,
    pub(crate) edge_removal_service_api: edge::OnNodeRemovalServiceApi<'a>,
}

impl NodeLifecycleView<'_> {
    pub fn create(&mut self, spec: &NodeSpec) -> Result<NodeId> {
        let id = self.node_service.create(
            self.node_facade_view.spec,
            self.node_facade_view.node,
            spec,
        )?;
        if let Some(ports_spec) = spec.ports().as_ref() {
            let mut port_state_view = PortStateViewMut {
                node_repo: self.node_facade_view.node,
                spec_repo: self.node_facade_view.spec,
                port_conn_repo: self.node_facade_view.port_connection,
                port_state_repo: self.node_facade_view.port_state,
                channel_repo: self.channel_repo,
                channel_spec_repo: self.channel_spec_repo,
                channel_service: self.channel_service,
            };

            for port_id in ports_spec.ids() {
                port_state_view.set_state(id, *port_id, PortState::new(PortSource::Internal))?;
            }
        }

        Ok(id)
    }

    pub fn remove(&mut self, spec: &NodeSpec, id: NodeId) -> Result<()> {
        let mut port_conn_view = PortConnectionViewMut {
            node_repo: self.node_facade_view.node,
            spec_repo: self.node_facade_view.spec,
            port_conn_repo: self.node_facade_view.port_connection,
            port_state_repo: self.node_facade_view.port_state,
            channel_repo: self.channel_repo,
            channel_spec_repo: self.channel_spec_repo,
            channel_service: self.channel_service,
        };
        port_conn_view.disconnect_all_ports(id)?;

        self.node_service.remove(spec, id);
        self.node_facade_view.node.remove(id);
        self.edge_removal_service_api.on_removal(id)?;

        if spec.has_params() {
            self.node_facade_view.parameter.remove(id);
        }
        Ok(())
    }
}

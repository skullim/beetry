use crate::{
    repository::{ChannelRepository, ChannelSpecRepository, NodeRepositoryFacadeViewMut},
    service::{channel::ChannelService, edge},
};
use anyhow::Result;
use beetry_editor_types::persistence::PortsStateMap;
use beetry_editor_types::{
    id::{NodeId, NodePortId, NodeSpecId, PortConnectionId},
    output::node::{PortSource, PortState},
    persistence::{NodeRecord, ParameterValues},
    spec::node::NodeSpec,
};

use super::{NodeService, PortConnectionViewMut, PortStateViewMut};

pub(crate) struct LoadNodeView<'s, 'r> {
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
        node: NodeRecord,
        param_value: Option<ParameterValues>,
        ports_state: Option<PortsStateMap>,
    ) -> Result<()> {
        self.node_service.load_node(
            self.node_facade_view.specs,
            self.node_facade_view.nodes,
            node.id,
            node.value.spec_id(),
        )?;
        if let Some(map) = ports_state {
            for (port_id, state) in map {
                self.load_port_state(node.id, port_id, state)?;
            }
        }

        if let Some(value) = param_value {
            self.load_parameters(node.id, value);
        }
        Ok(())
    }

    fn load_parameters(&mut self, id: NodeId, value: ParameterValues) {
        self.node_facade_view.parameters.create(id, value.params);
    }

    fn load_port_state(&mut self, node: NodeId, port: NodePortId, state: PortState) -> Result<()> {
        self.node_facade_view.ports.insert(node, port, state)?;
        Ok(())
    }

    pub(crate) fn load_port_connections(
        &mut self,
        conns: impl IntoIterator<Item = PortConnectionId>,
    ) -> Result<()> {
        for conn in conns {
            self.node_facade_view.port_connections.insert(conn)?;
        }
        Ok(())
    }

    pub(crate) fn load_spec(&mut self, id: NodeSpecId, spec: NodeSpec) -> Result<()> {
        self.node_service
            .load_spec(self.node_facade_view.specs, id, spec)
    }
}

pub(crate) struct NodeLifecycleView<'a> {
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
            self.node_facade_view.specs,
            self.node_facade_view.nodes,
            spec,
        )?;
        if let Some(ports_spec) = spec.ports() {
            let mut port_state_view = PortStateViewMut {
                node_repo: self.node_facade_view.nodes,
                spec_repo: self.node_facade_view.specs,
                port_conn_repo: self.node_facade_view.port_connections,
                port_state_repo: self.node_facade_view.ports,
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
            node_repo: self.node_facade_view.nodes,
            spec_repo: self.node_facade_view.specs,
            port_conn_repo: self.node_facade_view.port_connections,
            port_state_repo: self.node_facade_view.ports,
            channel_repo: self.channel_repo,
            channel_spec_repo: self.channel_spec_repo,
            channel_service: self.channel_service,
        };
        port_conn_view.disconnect_all_ports(id)?;

        self.node_service.remove(spec, id)?;
        self.node_facade_view.nodes.remove(id);
        self.edge_removal_service_api.on_removal(id)?;

        if spec.has_params() {
            self.node_facade_view.parameters.remove(id);
        }
        Ok(())
    }
}

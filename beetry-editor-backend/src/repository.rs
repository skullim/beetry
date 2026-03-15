use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::Hash,
    ops::AddAssign,
};

use anyhow::{Result, bail};
use beetry_editor_types::{
    id::{ChannelId, ChannelSpecId, EdgeId, NodeId, NodePortId, NodeSpecId, PortConnectionId},
    output::{
        channel::ChannelData,
        edge::NodeEdge,
        node::{Parameters, PortState},
        ui::{ChannelUiData, NodeUiData, PortConnectionUiData},
    },
    spec::{channel::ChannelSpec, node::NodeSpec},
};
use num_traits::One;

use crate::id::IdProvider;

pub struct EditorRepositoryView<'a> {
    pub node: NodeRepositoryFacadeView<'a>,
    pub edge: &'a EdgeRepository,
    pub channel: ChannelRepositoryFacadeView<'a>,
    pub ui: UiRepositoryFacadeView<'a>,
}

pub struct EditorRepositoryViewMut<'a> {
    pub node: NodeRepositoryFacadeViewMut<'a>,
    pub edge: &'a mut EdgeRepository,
    pub channel: ChannelRepositoryFacadeViewMut<'a>,
    pub ui: UiRepositoryFacadeViewMut<'a>,
}
#[derive(Debug, Default)]
pub struct EditorRepository {
    pub(crate) node: NodeRepository,
    pub(crate) node_spec: NodeSpecRepository,
    pub(crate) parameter: ParamValuesRepository,
    pub(crate) port_connection: PortConnectionRepository,
    pub(crate) port_state: PortStateRepository,
    pub(crate) edge: EdgeRepository,
    pub(crate) channel: ChannelRepository,
    pub(crate) channel_spec: ChannelSpecRepository,
    pub(crate) ui_node: NodeUiRepository,
    pub(crate) ui_channel: ChannelUiRepository,
    pub(crate) ui_port_connection: PortConnectionUiRepository,
}

impl EditorRepository {
    pub fn view(&self) -> EditorRepositoryView<'_> {
        EditorRepositoryView {
            node: NodeRepositoryFacadeView {
                node: &self.node,
                spec: &self.node_spec,
                parameter: &self.parameter,
                port_connection: &self.port_connection,
                port_state: &self.port_state,
            },
            edge: &self.edge,
            channel: ChannelRepositoryFacadeView {
                spec: &self.channel_spec,
                channel: &self.channel,
            },
            ui: UiRepositoryFacadeView {
                node: &self.ui_node,
                channel: &self.ui_channel,
                port_connection: &self.ui_port_connection,
            },
        }
    }

    pub fn view_mut(&mut self) -> EditorRepositoryViewMut<'_> {
        EditorRepositoryViewMut {
            node: NodeRepositoryFacadeViewMut {
                node: &mut self.node,
                spec: &mut self.node_spec,
                parameter: &mut self.parameter,
                port_connection: &mut self.port_connection,
                port_state: &mut self.port_state,
            },
            edge: &mut self.edge,
            channel: ChannelRepositoryFacadeViewMut {
                spec: &mut self.channel_spec,
                channel: &mut self.channel,
            },
            ui: UiRepositoryFacadeViewMut {
                node: &mut self.ui_node,
                channel: &mut self.ui_channel,
                port_connection: &mut self.ui_port_connection,
            },
        }
    }

    /// Instance should be initialized in default state, the interaction with
    /// concrete repositories should be managed by service layer.
    /// This saves ton of validation (to guarantee repositories are in correct
    /// state) that would be necessary to perform if Self would take the
    /// repositories in the constructor.
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy)]
pub struct NodeRepositoryFacadeView<'a> {
    pub node: &'a NodeRepository,
    pub spec: &'a NodeSpecRepository,
    pub parameter: &'a ParamValuesRepository,
    pub port_connection: &'a PortConnectionRepository,
    pub port_state: &'a PortStateRepository,
}

pub struct NodeRepositoryFacadeViewMut<'a> {
    pub node: &'a mut NodeRepository,
    pub spec: &'a mut NodeSpecRepository,
    pub parameter: &'a mut ParamValuesRepository,
    pub port_connection: &'a mut PortConnectionRepository,
    pub port_state: &'a mut PortStateRepository,
}

pub struct ChannelRepositoryFacadeView<'a> {
    pub spec: &'a ChannelSpecRepository,
    pub channel: &'a ChannelRepository,
}

pub struct ChannelRepositoryFacadeViewMut<'a> {
    pub spec: &'a mut ChannelSpecRepository,
    pub channel: &'a mut ChannelRepository,
}

// ui

pub struct UiRepositoryFacadeView<'a> {
    pub node: &'a NodeUiRepository,
    pub channel: &'a ChannelUiRepository,
    pub port_connection: &'a PortConnectionUiRepository,
}

pub struct UiRepositoryFacadeViewMut<'a> {
    pub node: &'a mut NodeUiRepository,
    pub channel: &'a mut ChannelUiRepository,
    pub port_connection: &'a mut PortConnectionUiRepository,
}

/// Service layer should guarantee that unique specs are stored.
#[derive(Debug)]
pub struct SpecRepository<I, S> {
    specs: HashMap<I, S>,
    id_provider: IdProvider<I>,
}
impl<I, S> Default for SpecRepository<I, S>
where
    I: Default,
{
    fn default() -> Self {
        Self {
            specs: <_>::default(),
            id_provider: <_>::default(),
        }
    }
}

impl<I, S> SpecRepository<I, S>
where
    I: Default + One + Hash + Eq + Display + Copy + AddAssign,
    S: Sized,
{
    pub fn create(&mut self, spec: S) -> I {
        let id = self
            .id_provider
            .next_available_id(|id| !self.specs.contains_key(id));
        self.specs.insert(id, spec);
        id
    }

    pub fn load(&mut self, id: I, spec: S) -> Result<()> {
        if self.specs.contains_key(&id) {
            bail!("cannot load spec {id} as there is already spec stored with the same id");
        }
        self.specs.insert(id, spec);
        Ok(())
    }

    pub fn remove(&mut self, id: I) -> Option<S> {
        self.specs.remove(&id)
    }

    pub fn spec(&self, id: I) -> Option<&S> {
        self.specs.get(&id)
    }

    pub fn ids(&self) -> impl Iterator<Item = &I> {
        self.specs.keys()
    }

    pub fn specs(&self) -> impl Iterator<Item = &S> {
        self.specs.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&I, &S)> {
        self.ids().zip(self.specs())
    }
}

pub type NodeSpecRepository = SpecRepository<NodeSpecId, NodeSpec>;

#[derive(Debug, Default)]
pub struct NodeRepository {
    nodes: HashMap<NodeId, NodeSpecId>,
    id_provider: IdProvider<NodeId>,
}

impl NodeRepository {
    pub fn create(&mut self, spec: NodeSpecId) -> NodeId {
        let id = self
            .id_provider
            .next_available_id(|id| !self.nodes.contains_key(id));
        self.nodes.insert(id, spec);
        id
    }

    pub fn load(&mut self, node: NodeId, spec: NodeSpecId) -> Result<()> {
        if self.nodes.contains_key(&node) {
            bail!("cannot load node {node} as there is already a node stored with the same id");
        }
        self.nodes.insert(node, spec);
        Ok(())
    }

    pub fn remove(&mut self, id: NodeId) -> Option<NodeSpecId> {
        self.nodes.remove(&id)
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    pub fn spec_id(&self, id: NodeId) -> Option<&NodeSpecId> {
        self.nodes.get(&id)
    }

    pub fn spec_ids(&self) -> impl Iterator<Item = &NodeSpecId> {
        self.nodes.values()
    }

    pub fn ids(&self) -> impl Iterator<Item = &NodeId> {
        self.nodes.keys()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&NodeId, &NodeSpecId)> {
        self.ids().zip(self.spec_ids())
    }
}

#[derive(Debug, Default)]
pub struct PortStateRepository {
    states: HashMap<(NodeId, NodePortId), PortState>,
}

impl PortStateRepository {
    pub fn insert(&mut self, node: NodeId, port: NodePortId, state: PortState) {
        self.states.insert((node, port), state);
    }

    pub fn state(&self, node: NodeId, port: NodePortId) -> Option<&PortState> {
        self.states.get(&(node, port))
    }
}

#[derive(Debug, Default)]
pub struct PortConnectionRepository {
    conns: HashMap<NodeId, HashMap<NodePortId, HashSet<ChannelId>>>,
}

impl PortConnectionRepository {
    pub fn insert(&mut self, conn: PortConnectionId) {
        self.conns
            .entry(conn.node_id)
            .or_default()
            .entry(conn.port_id)
            .or_default()
            .insert(conn.channel_id);
    }

    pub fn remove(&mut self, conn: PortConnectionId) {
        if let Some(port_conns) = self.conns.get_mut(&conn.node_id)
            && let Some(channel_ids) = port_conns.get_mut(&conn.port_id)
        {
            {
                channel_ids.remove(&conn.channel_id);
            }
        }
    }

    pub fn remove_port_conns(
        &mut self,
        node: NodeId,
        port: NodePortId,
    ) -> Option<impl IntoIterator<Item = ChannelId> + use<>> {
        if let Some(node_conns) = self.conns.get_mut(&node)
            && let Some(port_conns) = node_conns.remove(&port)
        {
            {
                return Some(port_conns.into_iter());
            }
        }
        None
    }

    pub fn remove_node_conns(
        &mut self,
        id: NodeId,
    ) -> Option<impl IntoIterator<Item = (NodePortId, ChannelId)> + use<>> {
        self.conns.remove(&id).map(|node_conns| {
            node_conns.into_iter().flat_map(|(port_id, channel_ids)| {
                channel_ids
                    .into_iter()
                    .map(move |channel_id| (port_id, channel_id))
            })
        })
    }

    pub fn conn_exists(&self, conn: PortConnectionId) -> bool {
        self.conns
            .get(&conn.node_id)
            .and_then(|ports| ports.get(&conn.port_id))
            .is_some_and(|channels| channels.contains(&conn.channel_id))
    }

    pub fn is_port_connected(&self, node: NodeId, port: NodePortId) -> bool {
        self.conns
            .get(&node)
            .and_then(|ports| ports.get(&port))
            .is_some_and(|channels| !channels.is_empty())
    }

    pub fn node_conns(&self, node: NodeId) -> impl Iterator<Item = (&NodePortId, &ChannelId)> {
        self.conns.get(&node).into_iter().flat_map(|ports| {
            ports.iter().flat_map(|(port_id, channels)| {
                channels.iter().map(move |channel_id| (port_id, channel_id))
            })
        })
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&NodeId, impl Iterator<Item = (&NodePortId, &ChannelId)>)> {
        self.conns.iter().map(|(node_id, ports)| {
            let it = ports.iter().flat_map(|(port_id, channels)| {
                channels.iter().map(move |channel_id| (port_id, channel_id))
            });
            (node_id, it)
        })
    }
}

/// Caller (service layer) has to assure that params are valid w.r.t. schema.
#[derive(Debug, Default)]
pub struct ParamValuesRepository {
    map: HashMap<NodeId, Parameters>,
}

impl ParamValuesRepository {
    pub fn create(&mut self, id: NodeId, value: Parameters) {
        self.map.insert(id, value);
    }

    pub fn remove(&mut self, id: NodeId) -> Option<Parameters> {
        self.map.remove(&id)
    }

    pub fn params(&self, id: NodeId) -> Option<&Parameters> {
        self.map.get(&id)
    }
}

#[derive(Debug, Default)]
pub struct EdgeRepository {
    edges: HashMap<EdgeId, NodeEdge>,
    id_provider: IdProvider<EdgeId>,
}

impl EdgeRepository {
    pub fn create(&mut self, edge: NodeEdge) -> EdgeId {
        let id = self
            .id_provider
            .next_available_id(|id| !self.edges.contains_key(id));
        self.edges.insert(id, edge);
        id
    }

    pub fn remove(&mut self, id: EdgeId) -> Option<NodeEdge> {
        self.edges.remove(&id)
    }

    pub fn edges(&self) -> impl Iterator<Item = &NodeEdge> {
        self.edges.values()
    }

    pub fn ids(&self) -> impl Iterator<Item = &EdgeId> {
        self.edges.keys()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&EdgeId, &NodeEdge)> {
        self.ids().zip(self.edges())
    }
}

pub type ChannelSpecRepository = SpecRepository<ChannelSpecId, ChannelSpec>;

#[derive(Debug, Default)]
pub struct ChannelRepository {
    channels: HashMap<ChannelId, ChannelData>,
    id_provider: IdProvider<ChannelId>,
}

impl ChannelRepository {
    pub fn create(&mut self, data: ChannelData) -> ChannelId {
        let id = self
            .id_provider
            .next_available_id(|id| !self.channels.contains_key(id));
        self.channels.insert(id, data);
        id
    }

    pub fn load(&mut self, id: ChannelId, data: ChannelData) -> Result<()> {
        if self.channels.contains_key(&id) {
            bail!("cannot load channel {id} as there is already a channel stored with the same id");
        }
        self.channels.insert(id, data);
        Ok(())
    }

    pub fn remove(&mut self, id: ChannelId) -> Option<ChannelData> {
        self.channels.remove(&id)
    }

    pub fn contains(&self, id: ChannelId) -> bool {
        self.channels.contains_key(&id)
    }

    pub fn data_mut(&mut self, id: ChannelId) -> Option<&mut ChannelData> {
        self.channels.get_mut(&id)
    }

    pub fn data(&self, id: ChannelId) -> Option<&ChannelData> {
        self.channels.get(&id)
    }

    pub fn channels(&self) -> impl Iterator<Item = &ChannelId> {
        self.channels.keys()
    }
}

// UI components are always backed up by other "real" entities, therefore no
// need to use new identifiers.
#[derive(Debug)]
pub struct UiRepository<I, D> {
    data: HashMap<I, D>,
}

impl<I, D> Default for UiRepository<I, D> {
    fn default() -> Self {
        Self {
            data: <_>::default(),
        }
    }
}

impl<I, D> UiRepository<I, D>
where
    I: Eq + Hash + Copy,
{
    pub fn create(&mut self, id: I, data: D) {
        self.data.insert(id, data);
    }

    pub fn remove(&mut self, id: I) -> Option<D> {
        self.data.remove(&id)
    }

    pub fn data(&self, id: I) -> Option<&D> {
        self.data.get(&id)
    }

    pub fn data_mut(&mut self, id: I) -> Option<&mut D> {
        self.data.get_mut(&id)
    }

    pub fn id_iter(&self) -> impl Iterator<Item = &I> {
        self.data.keys()
    }

    pub fn data_iter(&self) -> impl Iterator<Item = &D> {
        self.data.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&I, &D)> {
        self.id_iter().zip(self.data_iter())
    }
}

pub type NodeUiRepository = UiRepository<NodeId, NodeUiData>;
pub type ChannelUiRepository = UiRepository<ChannelId, ChannelUiData>;

pub type PortConnectionUiRepository = UiRepository<PortConnectionId, PortConnectionUiData>;

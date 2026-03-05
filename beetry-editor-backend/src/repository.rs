use crate::id::IdProvider;
use anyhow::{Result, bail};
use beetry_editor_types::{
    id::{ChannelId, ChannelSpecId, EdgeId, NodeId, NodePortId, NodeSpecId, PortConnectionId},
    output::{
        channel::ChannelData,
        edge::NodeEdge,
        node::{ParameterValue, Parameters, PortState},
        ui::{ChannelUiData, NodeUiData, PortConnectionUiData},
    },
    spec::{
        channel::ChannelSpec,
        node::{FieldName, NodeSpec},
    },
};
use getset::{Getters, MutGetters};
use num_traits::One;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::Hash,
    ops::AddAssign,
};

#[derive(Debug, Default, Getters, MutGetters)]
pub struct EditorRepository<NRF, ER, CRF, UR> {
    #[getset(get = "pub", get_mut = "pub")]
    node: NRF,
    #[getset(get = "pub", get_mut = "pub")]
    edge: ER,
    #[getset(get = "pub", get_mut = "pub")]
    channel: CRF,
    #[getset(get = "pub", get_mut = "pub")]
    ui: UR,
}

pub struct EditorRepositoryView<'a, NRF, ER, CRF, UR> {
    pub node: &'a NRF,
    pub edge: &'a ER,
    pub channel: &'a CRF,
    pub ui: &'a UR,
}

pub struct EditorRepositoryViewMut<'a, NRF, ER, CR, UR> {
    pub node: &'a mut NRF,
    pub edge: &'a mut ER,
    pub channel: &'a mut CR,
    pub ui: &'a mut UR,
}

impl<NRF, ER, CRF, UR> EditorRepository<NRF, ER, CRF, UR> {
    pub fn view(&self) -> EditorRepositoryView<'_, NRF, ER, CRF, UR> {
        EditorRepositoryView {
            node: &self.node,
            edge: &self.edge,
            channel: &self.channel,
            ui: &self.ui,
        }
    }

    pub fn view_mut(&mut self) -> EditorRepositoryViewMut<'_, NRF, ER, CRF, UR> {
        EditorRepositoryViewMut {
            node: &mut self.node,
            edge: &mut self.edge,
            channel: &mut self.channel,
            ui: &mut self.ui,
        }
    }
}

impl<NRF, ER, CRF, UR> EditorRepository<NRF, ER, CRF, UR>
where
    NRF: Default,
    ER: Default,
    CRF: Default,
    UR: Default,
{
    /// Instance should be initialized in default state, the interaction with concrete repositories
    /// should be managed by service layer.
    /// This saves ton of validation (to guarantee repositories are in correct state) that would be
    /// necessary to perform if Self would take the repositories in the constructor.
    pub fn new() -> Self {
        Self {
            node: NRF::default(),
            edge: ER::default(),
            channel: CRF::default(),
            ui: UR::default(),
        }
    }
}

pub trait NodeRepositoryFacadeConcept: Default {
    type NodeRepo: NodeRepositoryConcept;
    type SpecRepo: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>;

    type ParamValuesRepo: ParamValueRepositoryConcept;
    type PortConnectionRepo: PortConnectionRepositoryConcept;
    type PortStateRepo: PortStateRepositoryConcept;

    fn view(&self) -> NodeRepositoryFacadeView<'_, Self>
    where
        Self: Sized;

    fn view_mut(&mut self) -> NodeRepositoryFacadeViewMut<'_, Self>
    where
        Self: Sized;
}

pub struct NodeRepositoryFacadeView<'a, F: NodeRepositoryFacadeConcept> {
    pub nodes: &'a F::NodeRepo,
    pub specs: &'a F::SpecRepo,
    pub parameters: &'a F::ParamValuesRepo,
    pub port_connections: &'a F::PortConnectionRepo,
    pub ports: &'a F::PortStateRepo,
}

impl<F: NodeRepositoryFacadeConcept> Copy for NodeRepositoryFacadeView<'_, F> {}

impl<F: NodeRepositoryFacadeConcept> Clone for NodeRepositoryFacadeView<'_, F> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct NodeRepositoryFacadeViewMut<'a, F: NodeRepositoryFacadeConcept> {
    pub nodes: &'a mut F::NodeRepo,
    pub specs: &'a mut F::SpecRepo,
    pub parameters: &'a mut F::ParamValuesRepo,
    pub port_connections: &'a mut F::PortConnectionRepo,
    pub ports: &'a mut F::PortStateRepo,
}

#[derive(Default)]
pub struct NodeRepositoryFacade {
    node: NodeRepository,
    spec: NodeSpecRepository,
    parameter: ParamValuesRepository,
    port_connection: PortConnectionRepository,
    port: PortStateRepository,
}

impl NodeRepositoryFacade {
    pub fn new(
        node: NodeRepository,
        spec: NodeSpecRepository,
        parameter: ParamValuesRepository,
        port_connection: PortConnectionRepository,
        port: PortStateRepository,
    ) -> Self {
        Self {
            node,
            spec,
            parameter,
            port_connection,
            port,
        }
    }
}

impl NodeRepositoryFacadeConcept for NodeRepositoryFacade {
    type NodeRepo = NodeRepository;
    type SpecRepo = NodeSpecRepository;
    type ParamValuesRepo = ParamValuesRepository;
    type PortConnectionRepo = PortConnectionRepository;
    type PortStateRepo = PortStateRepository;

    fn view(&self) -> NodeRepositoryFacadeView<'_, Self> {
        NodeRepositoryFacadeView {
            nodes: &self.node,
            specs: &self.spec,
            parameters: &self.parameter,
            port_connections: &self.port_connection,
            ports: &self.port,
        }
    }

    fn view_mut(&mut self) -> NodeRepositoryFacadeViewMut<'_, Self> {
        NodeRepositoryFacadeViewMut {
            nodes: &mut self.node,
            specs: &mut self.spec,
            parameters: &mut self.parameter,
            port_connections: &mut self.port_connection,
            ports: &mut self.port,
        }
    }
}

pub trait ChannelRepositoryFacadeConcept: Default {
    type SpecRepo: SpecRepositoryConcept<Spec = ChannelSpec, SpecId = ChannelSpecId>;
    type DataRepo: ChannelRepositoryConcept;

    fn view(&self) -> ChannelRepositoryFacadeView<'_, Self>
    where
        Self: Sized;

    fn view_mut(&mut self) -> ChannelRepositoryFacadeViewMut<'_, Self>
    where
        Self: Sized;
}

pub struct ChannelRepositoryFacadeView<'a, F: ChannelRepositoryFacadeConcept> {
    pub spec: &'a F::SpecRepo,
    pub channel: &'a F::DataRepo,
}

pub struct ChannelRepositoryFacadeViewMut<'a, F: ChannelRepositoryFacadeConcept> {
    pub spec: &'a mut F::SpecRepo,
    pub channel: &'a mut F::DataRepo,
}
#[derive(Default)]
pub struct ChannelRepositoryFacade {
    data: ChannelRepository,
    spec: ChannelSpecRepository,
}

impl ChannelRepositoryFacade {
    pub fn new(data: ChannelRepository, spec: ChannelSpecRepository) -> Self {
        Self { data, spec }
    }
}

impl ChannelRepositoryFacadeConcept for ChannelRepositoryFacade {
    type DataRepo = ChannelRepository;
    type SpecRepo = ChannelSpecRepository;

    fn view(&self) -> ChannelRepositoryFacadeView<'_, Self>
    where
        Self: Sized,
    {
        ChannelRepositoryFacadeView {
            channel: &self.data,
            spec: &self.spec,
        }
    }

    fn view_mut(&mut self) -> ChannelRepositoryFacadeViewMut<'_, Self>
    where
        Self: Sized,
    {
        ChannelRepositoryFacadeViewMut {
            channel: &mut self.data,
            spec: &mut self.spec,
        }
    }
}

// ui

pub trait UiRepositoryFacadeConcept: Default {
    type UiNodeRepo: UiRepositoryConcept<Id = NodeId, Data = NodeUiData>;
    type UiChannelRepo: UiRepositoryConcept<Id = ChannelId, Data = ChannelUiData>;
    type UiPortConnectionRepo: UiRepositoryConcept<Id = PortConnectionId, Data = PortConnectionUiData>;

    fn view(&self) -> UiRepositoryFacadeView<'_, Self>
    where
        Self: Sized;

    fn view_mut(&mut self) -> UiRepositoryFacadeViewMut<'_, Self>
    where
        Self: Sized;
}

pub struct UiRepositoryFacadeView<'a, F: UiRepositoryFacadeConcept> {
    pub node: &'a F::UiNodeRepo,
    pub channel: &'a F::UiChannelRepo,
    pub port_connection: &'a F::UiPortConnectionRepo,
}

pub struct UiRepositoryFacadeViewMut<'a, F: UiRepositoryFacadeConcept> {
    pub node: &'a mut F::UiNodeRepo,
    pub channel: &'a mut F::UiChannelRepo,
    pub port_connection: &'a mut F::UiPortConnectionRepo,
}
#[derive(Default)]
pub struct UiRepositoryFacade {
    node: NodeUiRepository,
    channel: ChannelUiRepository,
    port_connection: PortConnectionUiRepository,
}

impl UiRepositoryFacade {
    pub fn new(
        node: NodeUiRepository,
        channel: ChannelUiRepository,
        port_connection: PortConnectionUiRepository,
    ) -> Self {
        Self {
            node,
            channel,
            port_connection,
        }
    }
}

impl UiRepositoryFacadeConcept for UiRepositoryFacade {
    type UiNodeRepo = NodeUiRepository;
    type UiChannelRepo = ChannelUiRepository;
    type UiPortConnectionRepo = PortConnectionUiRepository;

    fn view(&self) -> UiRepositoryFacadeView<'_, Self>
    where
        Self: Sized,
    {
        UiRepositoryFacadeView {
            node: &self.node,
            channel: &self.channel,
            port_connection: &self.port_connection,
        }
    }

    fn view_mut(&mut self) -> UiRepositoryFacadeViewMut<'_, Self>
    where
        Self: Sized,
    {
        UiRepositoryFacadeViewMut {
            node: &mut self.node,
            channel: &mut self.channel,
            port_connection: &mut self.port_connection,
        }
    }
}

/// Service layer should guarantee that unique specs are stored
pub trait SpecRepositoryConcept: Default {
    type Spec;
    type SpecId;

    fn create(&mut self, spec: Self::Spec) -> Result<Self::SpecId>;
    fn load(&mut self, id: Self::SpecId, spec: Self::Spec) -> Result<()>;

    fn remove(&mut self, id: Self::SpecId) -> Option<Self::Spec>;

    fn spec(&self, id: Self::SpecId) -> Option<&Self::Spec>;
    fn specs(&self) -> impl Iterator<Item = &Self::Spec>;

    fn ids(&self) -> impl Iterator<Item = &Self::SpecId>;
    // provided methods
    fn iter(&self) -> impl Iterator<Item = (&Self::SpecId, &Self::Spec)> {
        self.ids().zip(self.specs())
    }
}

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

impl<I, S> SpecRepositoryConcept for SpecRepository<I, S>
where
    I: Default + One + Hash + Eq + Display + Copy + AddAssign,
    S:,
{
    type Spec = S;
    type SpecId = I;

    fn create(&mut self, spec: S) -> Result<I> {
        let id = self
            .id_provider
            .next_available_id(|id| !self.specs.contains_key(id));
        self.specs.insert(id, spec);
        Ok(id)
    }

    fn load(&mut self, id: I, spec: S) -> Result<()> {
        if self.specs.contains_key(&id) {
            bail!("cannot load spec {id} as there is already spec stored with the same id");
        }
        self.specs.insert(id, spec);
        Ok(())
    }

    fn remove(&mut self, id: I) -> Option<S> {
        self.specs.remove(&id)
    }

    fn spec(&self, id: I) -> Option<&S> {
        self.specs.get(&id)
    }

    fn ids(&self) -> impl Iterator<Item = &Self::SpecId> {
        self.specs.keys()
    }

    fn specs(&self) -> impl Iterator<Item = &Self::Spec> {
        self.specs.values()
    }
}

pub type NodeSpecRepository = SpecRepository<NodeSpecId, NodeSpec>;

pub trait NodeRepositoryConcept: Default {
    fn create(&mut self, spec_id: NodeSpecId) -> Result<NodeId>;
    fn load(&mut self, id: NodeId, spec_id: NodeSpecId) -> Result<()>;

    fn remove(&mut self, id: NodeId) -> Option<NodeSpecId>;

    fn contains(&self, id: &NodeId) -> bool;

    fn spec_id(&self, id: &NodeId) -> Option<&NodeSpecId>;
    fn spec_ids(&self) -> impl Iterator<Item = &NodeSpecId>;

    fn ids(&self) -> impl Iterator<Item = &NodeId>;
    // provided methods
    fn iter(&self) -> impl Iterator<Item = (&NodeId, &NodeSpecId)> {
        self.ids().zip(self.spec_ids())
    }
}

#[derive(Debug, Default)]
pub struct NodeRepository {
    nodes: HashMap<NodeId, NodeSpecId>,
    id_provider: IdProvider<NodeId>,
}

impl NodeRepositoryConcept for NodeRepository {
    fn create(&mut self, spec: NodeSpecId) -> Result<NodeId> {
        let id = self
            .id_provider
            .next_available_id(|id| !self.nodes.contains_key(id));
        self.nodes.insert(id, spec);
        Ok(id)
    }

    fn load(&mut self, node: NodeId, spec: NodeSpecId) -> Result<()> {
        if self.nodes.contains_key(&node) {
            bail!("cannot load node {node} as there is already a node stored with the same id");
        }
        self.nodes.insert(node, spec);
        Ok(())
    }

    fn remove(&mut self, id: NodeId) -> Option<NodeSpecId> {
        self.nodes.remove(&id)
    }

    fn contains(&self, id: &NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    fn spec_id(&self, id: &NodeId) -> Option<&NodeSpecId> {
        self.nodes.get(id)
    }

    fn spec_ids(&self) -> impl Iterator<Item = &NodeSpecId> {
        self.nodes.values()
    }

    fn ids(&self) -> impl Iterator<Item = &NodeId> {
        self.nodes.keys()
    }
}

pub trait PortStateRepositoryConcept: Default {
    fn insert(&mut self, node: NodeId, port: NodePortId, state: PortState) -> Result<()>;

    fn remove(&mut self, node: NodeId, port: NodePortId) -> Option<PortState>;

    fn state(&self, node: NodeId, port: NodePortId) -> Option<&PortState>;
    fn state_mut(&mut self, node: NodeId, port: NodePortId) -> Option<&mut PortState>;
}

#[derive(Default)]
pub struct PortStateRepository {
    states: HashMap<(NodeId, NodePortId), PortState>,
}

impl PortStateRepositoryConcept for PortStateRepository {
    fn insert(&mut self, node: NodeId, port: NodePortId, state: PortState) -> Result<()> {
        self.states.insert((node, port), state);
        Ok(())
    }

    fn remove(&mut self, node: NodeId, port: NodePortId) -> Option<PortState> {
        self.states.remove(&(node, port))
    }

    fn state(&self, node: NodeId, port: NodePortId) -> Option<&PortState> {
        self.states.get(&(node, port))
    }

    fn state_mut(&mut self, node: NodeId, port: NodePortId) -> Option<&mut PortState> {
        self.states.get_mut(&(node, port))
    }
}

pub trait PortConnectionRepositoryConcept: Default {
    fn insert(&mut self, conn: PortConnectionId) -> Result<()>;

    fn remove(&mut self, conn: PortConnectionId);
    fn remove_port_conns(
        &mut self,
        node: NodeId,
        port: NodePortId,
    ) -> Option<impl IntoIterator<Item = ChannelId> + use<Self>>;
    fn remove_node_conns(
        &mut self,
        id: NodeId,
    ) -> Option<impl IntoIterator<Item = (NodePortId, ChannelId)> + use<Self>>;

    fn conn_exists(&self, conn: PortConnectionId) -> bool;
    fn is_port_connected(&self, node: NodeId, port: NodePortId) -> bool;
    fn node_conns(&self, node: NodeId) -> impl Iterator<Item = (&NodePortId, &ChannelId)>;

    fn iter(
        &self,
    ) -> impl Iterator<Item = (&NodeId, impl Iterator<Item = (&NodePortId, &ChannelId)>)>;
}

#[derive(Default)]
pub struct PortConnectionRepository {
    conns: HashMap<NodeId, HashMap<NodePortId, HashSet<ChannelId>>>,
}

impl PortConnectionRepositoryConcept for PortConnectionRepository {
    fn insert(&mut self, conn: PortConnectionId) -> Result<()> {
        self.conns
            .entry(conn.node_id)
            .or_default()
            .entry(conn.port_id)
            .or_default()
            .insert(conn.channel_id);
        Ok(())
    }

    fn remove(&mut self, conn: PortConnectionId) {
        if let Some(port_conns) = self.conns.get_mut(&conn.node_id)
            && let Some(channel_ids) = port_conns.get_mut(&conn.port_id)
        {
            {
                channel_ids.remove(&conn.channel_id);
            }
        }
    }

    fn remove_port_conns(
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

    fn remove_node_conns(
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

    fn conn_exists(&self, conn: PortConnectionId) -> bool {
        self.conns
            .get(&conn.node_id)
            .and_then(|ports| ports.get(&conn.port_id))
            .is_some_and(|channels| channels.contains(&conn.channel_id))
    }

    fn is_port_connected(&self, node: NodeId, port: NodePortId) -> bool {
        self.conns
            .get(&node)
            .and_then(|ports| ports.get(&port))
            .is_some_and(|channels| !channels.is_empty())
    }

    fn node_conns(&self, node: NodeId) -> impl Iterator<Item = (&NodePortId, &ChannelId)> {
        self.conns.get(&node).into_iter().flat_map(|ports| {
            ports.iter().flat_map(|(port_id, channels)| {
                channels.iter().map(move |channel_id| (port_id, channel_id))
            })
        })
    }

    fn iter(
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

/// caller (service layer) has to assure that params are valid w.r.t. schema
pub trait ParamValueRepositoryConcept: Default {
    fn create(&mut self, id: NodeId, value: Parameters);

    fn remove(&mut self, id: NodeId) -> Option<Parameters>;

    fn param_value_mut(
        &mut self,
        id: NodeId,
        field_name: &FieldName,
    ) -> Option<&mut ParameterValue>;

    fn params(&self, id: NodeId) -> Option<&Parameters>;

    fn iter(&self) -> impl Iterator<Item = (&NodeId, &Parameters)>;
}

#[derive(Default)]
pub struct ParamValuesRepository {
    map: HashMap<NodeId, Parameters>,
}

impl ParamValueRepositoryConcept for ParamValuesRepository {
    fn create(&mut self, id: NodeId, value: Parameters) {
        self.map.insert(id, value);
    }

    fn remove(&mut self, id: NodeId) -> Option<Parameters> {
        self.map.remove(&id)
    }

    fn param_value_mut(
        &mut self,
        id: NodeId,
        field_name: &FieldName,
    ) -> Option<&mut ParameterValue> {
        self.map.get_mut(&id)?.get_mut(field_name)
    }

    fn params(&self, id: NodeId) -> Option<&Parameters> {
        self.map.get(&id)
    }

    fn iter(&self) -> impl Iterator<Item = (&NodeId, &Parameters)> {
        self.map.iter()
    }
}

pub trait EdgeRepositoryConcept: Default {
    fn create(&mut self, edge: NodeEdge) -> Result<EdgeId>;
    // no need for load method, edges are internal constructs that are not exposed outside

    fn remove(&mut self, id: &EdgeId) -> Option<NodeEdge>;

    fn edges(&self) -> impl Iterator<Item = &NodeEdge>;
    fn ids(&self) -> impl Iterator<Item = &EdgeId>;

    // provided methods
    fn iter(&self) -> impl Iterator<Item = (&EdgeId, &NodeEdge)> {
        self.ids().zip(self.edges())
    }
}

#[derive(Default)]
pub struct EdgeRepository {
    edges: HashMap<EdgeId, NodeEdge>,
    id_provider: IdProvider<EdgeId>,
}

impl EdgeRepositoryConcept for EdgeRepository {
    fn create(&mut self, edge: NodeEdge) -> Result<EdgeId> {
        let id = self
            .id_provider
            .next_available_id(|id| !self.edges.contains_key(id));
        self.edges.insert(id, edge);
        Ok(id)
    }

    fn remove(&mut self, id: &EdgeId) -> Option<NodeEdge> {
        self.edges.remove(id)
    }

    fn edges(&self) -> impl Iterator<Item = &NodeEdge> {
        self.edges.values()
    }

    fn ids(&self) -> impl Iterator<Item = &EdgeId> {
        self.edges.keys()
    }
}

pub trait ChannelRepositoryConcept: Default {
    fn create(&mut self, data: ChannelData) -> Result<ChannelId>;
    fn load(&mut self, id: ChannelId, data: ChannelData) -> Result<()>;

    fn remove(&mut self, id: ChannelId) -> Option<ChannelData>;

    fn contains(&self, id: &ChannelId) -> bool;

    fn data(&self, id: ChannelId) -> Option<&ChannelData>;
    fn data_mut(&mut self, id: ChannelId) -> Option<&mut ChannelData>;

    fn channels(&self) -> impl Iterator<Item = &ChannelId>;
}

pub type ChannelSpecRepository = SpecRepository<ChannelSpecId, ChannelSpec>;

#[derive(Default)]
pub struct ChannelRepository {
    channels: HashMap<ChannelId, ChannelData>,
    id_provider: IdProvider<ChannelId>,
}

impl ChannelRepositoryConcept for ChannelRepository {
    fn create(&mut self, data: ChannelData) -> Result<ChannelId> {
        let id = self
            .id_provider
            .next_available_id(|id| !self.channels.contains_key(id));
        self.channels.insert(id, data);
        Ok(id)
    }

    fn load(&mut self, id: ChannelId, data: ChannelData) -> Result<()> {
        if self.channels.contains_key(&id) {
            bail!("cannot load channel {id} as there is already a channel stored with the same id");
        }
        self.channels.insert(id, data);
        Ok(())
    }

    fn remove(&mut self, id: ChannelId) -> Option<ChannelData> {
        self.channels.remove(&id)
    }

    fn contains(&self, id: &ChannelId) -> bool {
        self.channels.contains_key(id)
    }

    fn data_mut(&mut self, id: ChannelId) -> Option<&mut ChannelData> {
        self.channels.get_mut(&id)
    }

    fn data(&self, id: ChannelId) -> Option<&ChannelData> {
        self.channels.get(&id)
    }

    fn channels(&self) -> impl Iterator<Item = &ChannelId> {
        self.channels.keys()
    }
}

//UI components are always backed up by other "real" entities, therefore no need to use new identifiers
pub trait UiRepositoryConcept: Default {
    type Id;
    type Data;

    fn create(&mut self, id: Self::Id, data: Self::Data) -> Result<()>;
    fn remove(&mut self, id: Self::Id) -> Option<Self::Data>;

    fn data(&self, id: Self::Id) -> Option<&Self::Data>;
    fn data_mut(&mut self, id: Self::Id) -> Option<&mut Self::Data>;

    fn id_iter(&self) -> impl Iterator<Item = &Self::Id>;
    fn data_iter(&self) -> impl Iterator<Item = &Self::Data>;

    // provided methods
    fn iter(&self) -> impl Iterator<Item = (&Self::Id, &Self::Data)> {
        self.id_iter().zip(self.data_iter())
    }
}

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

impl<I, D> UiRepositoryConcept for UiRepository<I, D>
where
    I: Eq + Hash,
{
    type Id = I;
    type Data = D;

    fn create(&mut self, id: Self::Id, data: Self::Data) -> Result<()> {
        self.data.insert(id, data);
        Ok(())
    }

    fn remove(&mut self, id: Self::Id) -> Option<Self::Data> {
        self.data.remove(&id)
    }

    fn data(&self, id: Self::Id) -> Option<&Self::Data> {
        self.data.get(&id)
    }

    fn data_mut(&mut self, id: Self::Id) -> Option<&mut Self::Data> {
        self.data.get_mut(&id)
    }

    fn id_iter(&self) -> impl Iterator<Item = &Self::Id> {
        self.data.keys()
    }

    fn data_iter(&self) -> impl Iterator<Item = &Self::Data> {
        self.data.values()
    }
}

pub type NodeUiRepository = UiRepository<NodeId, NodeUiData>;
pub type ChannelUiRepository = UiRepository<ChannelId, ChannelUiData>;

pub type PortConnectionUiRepository = UiRepository<PortConnectionId, PortConnectionUiData>;

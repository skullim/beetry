use std::{collections::HashMap, fmt::Display, hash::Hash, ops::AddAssign};

use crate::{
    domain::models::{
        ChannelData, ChannelId, ChannelSpecId, ChannelUiData, EdgeId, NodeEdge, NodePortConnection,
        NodePortId, NodeSpec, NodeSpecId, NodeUiData,
    },
    id::IdProvider,
};

use super::models::NodeId;
use beetry_plugin_types::channel::ChannelSpec;
use beetry_reconstruction_types::{channel::ChannelConfig, parameter::Parameters};

use anyhow::{Result, bail};
use getset::{Getters, MutGetters};
use num_traits::One;

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
    pub ports: &'a F::PortStateRepo,
}

impl<'a, F: NodeRepositoryFacadeConcept> Copy for NodeRepositoryFacadeView<'a, F> {}

impl<'a, F: NodeRepositoryFacadeConcept> Clone for NodeRepositoryFacadeView<'a, F> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct NodeRepositoryFacadeViewMut<'a, F: NodeRepositoryFacadeConcept> {
    pub nodes: &'a mut F::NodeRepo,
    pub specs: &'a mut F::SpecRepo,
    pub parameters: &'a mut F::ParamValuesRepo,
    pub ports: &'a mut F::PortStateRepo,
}

#[derive(Default)]
pub struct NodeRepositoryFacade {
    node: NodeRepository,
    spec: NodeSpecRepository,
    parameter: ParamValuesRepository,
    port: PortStateRepository,
}

impl NodeRepositoryFacade {
    pub fn new(
        node: NodeRepository,
        spec: NodeSpecRepository,
        parameter: ParamValuesRepository,
        port: PortStateRepository,
    ) -> Self {
        Self {
            node,
            spec,
            parameter,
            port,
        }
    }
}

impl NodeRepositoryFacadeConcept for NodeRepositoryFacade {
    type NodeRepo = NodeRepository;
    type SpecRepo = NodeSpecRepository;
    type ParamValuesRepo = ParamValuesRepository;
    type PortStateRepo = PortStateRepository;

    fn view(&self) -> NodeRepositoryFacadeView<'_, Self> {
        NodeRepositoryFacadeView {
            nodes: &self.node,
            specs: &self.spec,
            parameters: &self.parameter,
            ports: &self.port,
        }
    }

    fn view_mut(&mut self) -> NodeRepositoryFacadeViewMut<'_, Self> {
        NodeRepositoryFacadeViewMut {
            nodes: &mut self.node,
            specs: &mut self.spec,
            parameters: &mut self.parameter,
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
}

pub struct UiRepositoryFacadeViewMut<'a, F: UiRepositoryFacadeConcept> {
    pub node: &'a mut F::UiNodeRepo,
    pub channel: &'a mut F::UiChannelRepo,
}
#[derive(Default)]
pub struct UiRepositoryFacade {
    node: UiRepository<NodeId, NodeUiData>,
    channel: UiRepository<ChannelId, ChannelUiData>,
}

impl UiRepositoryFacade {
    pub fn new(
        node: UiRepository<NodeId, NodeUiData>,
        channel: UiRepository<ChannelId, ChannelUiData>,
    ) -> Self {
        Self { node, channel }
    }
}

impl UiRepositoryFacadeConcept for UiRepositoryFacade {
    type UiNodeRepo = UiRepository<NodeId, NodeUiData>;
    type UiChannelRepo = UiRepository<ChannelId, ChannelUiData>;

    fn view(&self) -> UiRepositoryFacadeView<'_, Self>
    where
        Self: Sized,
    {
        UiRepositoryFacadeView {
            node: &self.node,
            channel: &self.channel,
        }
    }

    fn view_mut(&mut self) -> UiRepositoryFacadeViewMut<'_, Self>
    where
        Self: Sized,
    {
        UiRepositoryFacadeViewMut {
            node: &mut self.node,
            channel: &mut self.channel,
        }
    }
}

/// Service layer should guarantee that no same Specs are stored
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

    fn remove(&mut self, id: NodeId) -> Result<()>;

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

    fn remove(&mut self, id: NodeId) -> Result<()> {
        self.nodes.remove(&id);
        Ok(())
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
    fn create(&mut self, node: NodeId, port: NodePortId, conn: NodePortConnection) -> Result<()>;

    fn remove(&mut self, node: NodeId, port: NodePortId) -> Option<NodePortConnection>;

    fn state(&self, node: NodeId, port: NodePortId) -> Option<&NodePortConnection>;
    fn state_mut(&mut self, node: NodeId, port: NodePortId) -> Option<&mut NodePortConnection>;

    fn port_iter(&self, node: NodeId) -> impl Iterator<Item = (&NodePortId, &NodePortConnection)>;
}

#[derive(Default)]
pub struct PortStateRepository {
    connections: HashMap<(NodeId, NodePortId), NodePortConnection>,
}

impl PortStateRepositoryConcept for PortStateRepository {
    fn create(&mut self, node: NodeId, port: NodePortId, conn: NodePortConnection) -> Result<()> {
        self.connections.insert((node, port), conn);
        Ok(())
    }

    fn remove(&mut self, node: NodeId, port: NodePortId) -> Option<NodePortConnection> {
        self.connections.remove(&(node, port))
    }

    fn state(&self, node: NodeId, port: NodePortId) -> Option<&NodePortConnection> {
        self.connections.get(&(node, port))
    }

    fn state_mut(&mut self, node: NodeId, port: NodePortId) -> Option<&mut NodePortConnection> {
        self.connections.get_mut(&(node, port))
    }

    fn port_iter(&self, node: NodeId) -> impl Iterator<Item = (&NodePortId, &NodePortConnection)> {
        self.connections
            .iter()
            .filter(move |((node_id, _), _)| *node_id == node)
            .map(|((_, port_id), v)| (port_id, v))
    }
}

/// caller (service layer) has to assure that params are valid w.r.t. schema
pub trait ParamValueRepositoryConcept: Default {
    fn create(&mut self, id: NodeId, params: Parameters) -> Result<()>;
    fn remove(&mut self, id: NodeId) -> Result<()>;

    fn value(&self, id: NodeId) -> Option<&Parameters>;
    fn value_mut(&mut self, id: NodeId) -> Option<&mut Parameters>;
}

#[derive(Default)]
pub struct ParamValuesRepository {
    params: HashMap<NodeId, Parameters>,
}

impl ParamValueRepositoryConcept for ParamValuesRepository {
    fn create(&mut self, id: NodeId, params: Parameters) -> Result<()> {
        self.params.insert(id, params);
        Ok(())
    }

    fn remove(&mut self, id: NodeId) -> Result<()> {
        self.params.remove(&id);
        Ok(())
    }

    fn value(&self, id: NodeId) -> Option<&Parameters> {
        self.params.get(&id)
    }

    fn value_mut(&mut self, id: NodeId) -> Option<&mut Parameters> {
        self.params.get_mut(&id)
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

    fn data_iter(&self) -> impl Iterator<Item = &ChannelData>;
    fn channels(&self) -> impl Iterator<Item = &ChannelId>;
}

impl ChannelData {
    pub fn new(spec_id: ChannelSpecId, config: ChannelConfig) -> Self {
        Self { spec_id, config }
    }
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

    fn data_iter(&self) -> impl Iterator<Item = &ChannelData> {
        self.channels.values()
    }

    fn channels(&self) -> impl Iterator<Item = &ChannelId> {
        self.channels.keys()
    }
}

pub trait UiRepositoryConcept: Default {
    type Id;
    type Data;

    fn create(&mut self, id: Self::Id, data: Self::Data) -> Result<()>;
    fn remove(&mut self, id: Self::Id) -> Result<()>;

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

    fn remove(&mut self, id: Self::Id) -> Result<()> {
        self.data.remove(&id);
        Ok(())
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

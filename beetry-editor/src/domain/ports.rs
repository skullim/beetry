use std::{collections::HashMap, fmt::Display, hash::Hash, ops::AddAssign};

use crate::{
    domain::models::{
        ChannelId, ChannelPosition, ChannelSpecId, EdgeId, NodeEdge, NodePortConnection,
        NodePortId, NodeSpec, NodeSpecId,
    },
    id::IdProvider,
};

use super::models::{NodeId, NodePosition};
use beetry_serde::{
    de::{channel::ChannelConfig, parameter::Parameters},
    ser::channel::ChannelSpec,
};

use anyhow::{Result, anyhow, bail};
use getset::{Getters, MutGetters};
use num_traits::One;
use serde_value::Value;

#[derive(Debug, Default, Getters, MutGetters)]
pub struct EditorRepository<NRF, ER, CRF> {
    #[getset(get = "pub", get_mut = "pub")]
    node: NRF,
    #[getset(get = "pub", get_mut = "pub")]
    edge: ER,
    #[getset(get = "pub", get_mut = "pub")]
    channel: CRF,
}

pub struct EditorRepositoryView<'a, NRF, ER, CRF> {
    pub node: &'a NRF,
    pub edge: &'a ER,
    pub channel: &'a CRF,
}

pub struct EditorRepositoryViewMut<'a, NRF, ER, CR> {
    pub node: &'a mut NRF,
    pub edge: &'a mut ER,
    pub channel: &'a mut CR,
}

impl<NRF, ER, CRF> EditorRepository<NRF, ER, CRF> {
    pub fn view(&self) -> EditorRepositoryView<'_, NRF, ER, CRF> {
        EditorRepositoryView {
            node: &self.node,
            edge: &self.edge,
            channel: &self.channel,
        }
    }

    pub fn view_mut(&mut self) -> EditorRepositoryViewMut<'_, NRF, ER, CRF> {
        EditorRepositoryViewMut {
            node: &mut self.node,
            edge: &mut self.edge,
            channel: &mut self.channel,
        }
    }
}

impl<NRF, ER, CRF> EditorRepository<NRF, ER, CRF>
where
    NRF: Default,
    ER: Default,
    CRF: Default,
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
        }
    }
}

pub trait NodeRepositoryFacadeConcept: Default {
    type NodeRepo: NodeRepositoryConcept;
    type SpecRepo: SpecRepositoryConcept<Spec = NodeSpec, SpecId = NodeSpecId>;

    type PositionRepo: NodePositionRepositoryConcept;
    type ParamValuesRepo: ParamValuesRepositoryConcept;
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
    pub positions: &'a F::PositionRepo,
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
    pub positions: &'a mut F::PositionRepo,
    pub parameters: &'a mut F::ParamValuesRepo,
    pub ports: &'a mut F::PortStateRepo,
}

#[derive(Default)]
pub struct NodeRepositoryFacade {
    node: NodeRepository,
    spec: NodeSpecRepository,
    position: NodePositionRepository,
    parameter: ParamValuesRepository,
    port: PortStateRepository,
}

impl NodeRepositoryFacade {
    pub fn new(
        node: NodeRepository,
        spec: NodeSpecRepository,
        position: NodePositionRepository,
        parameter: ParamValuesRepository,
        port: PortStateRepository,
    ) -> Self {
        Self {
            node,
            spec,
            position,
            parameter,
            port,
        }
    }
}

impl NodeRepositoryFacadeConcept for NodeRepositoryFacade {
    type NodeRepo = NodeRepository;
    type SpecRepo = NodeSpecRepository;
    type PositionRepo = NodePositionRepository;
    type ParamValuesRepo = ParamValuesRepository;
    type PortStateRepo = PortStateRepository;

    fn view(&self) -> NodeRepositoryFacadeView<'_, Self> {
        NodeRepositoryFacadeView {
            nodes: &self.node,
            specs: &self.spec,
            positions: &self.position,
            parameters: &self.parameter,
            ports: &self.port,
        }
    }

    fn view_mut(&mut self) -> NodeRepositoryFacadeViewMut<'_, Self> {
        NodeRepositoryFacadeViewMut {
            nodes: &mut self.node,
            specs: &mut self.spec,
            positions: &mut self.position,
            parameters: &mut self.parameter,
            ports: &mut self.port,
        }
    }
}

pub struct ChannelRepositoryFacadeView<'a, F: ChannelRepositoryFacadeConcept> {
    pub spec: &'a F::SpecRepo,
    pub data: &'a F::DataRepo,
}

pub struct ChannelRepositoryFacadeViewMut<'a, F: ChannelRepositoryFacadeConcept> {
    pub spec: &'a mut F::SpecRepo,
    pub data: &'a mut F::DataRepo,
}

pub trait ChannelRepositoryFacadeConcept: Default {
    type SpecRepo: SpecRepositoryConcept<Spec = ChannelSpec, SpecId = ChannelSpecId>;
    type DataRepo: ChannelDataRepositoryConcept;

    fn view(&self) -> ChannelRepositoryFacadeView<'_, Self>
    where
        Self: Sized;

    fn view_mut(&mut self) -> ChannelRepositoryFacadeViewMut<'_, Self>
    where
        Self: Sized;
}

#[derive(Default)]
pub struct ChannelRepositoryFacade {
    data: ChannelDataRepository,
    spec: ChannelSpecRepository,
}

impl ChannelRepositoryFacade {
    pub fn new(data: ChannelDataRepository, spec: ChannelSpecRepository) -> Self {
        Self { data, spec }
    }
}

impl ChannelRepositoryFacadeConcept for ChannelRepositoryFacade {
    type DataRepo = ChannelDataRepository;
    type SpecRepo = ChannelSpecRepository;

    fn view(&self) -> ChannelRepositoryFacadeView<'_, Self>
    where
        Self: Sized,
    {
        ChannelRepositoryFacadeView {
            data: &self.data,
            spec: &self.spec,
        }
    }

    fn view_mut(&mut self) -> ChannelRepositoryFacadeViewMut<'_, Self>
    where
        Self: Sized,
    {
        ChannelRepositoryFacadeViewMut {
            data: &mut self.data,
            spec: &mut self.spec,
        }
    }
}

/// Service layer should guarantee that no same Specs are stored
pub trait SpecRepositoryConcept: Default {
    type Spec;
    type SpecId;

    fn create(&mut self, spec: Self::Spec) -> Result<Self::SpecId>;
    /// Return error when id already used
    fn load(&mut self, id: Self::SpecId, spec: Self::Spec) -> Result<()>;

    fn remove(&mut self, id: Self::SpecId) -> Option<Self::Spec>;

    fn spec(&self, id: Self::SpecId) -> Option<&Self::Spec>;
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
}

pub type NodeSpecRepository = SpecRepository<NodeSpecId, NodeSpec>;
pub type ChannelSpecRepository = SpecRepository<ChannelSpecId, ChannelSpec>;

pub trait NodeRepositoryConcept: Default {
    fn create(&mut self, spec: NodeSpecId) -> Result<NodeId>;
    /// Return error when id already used
    fn load(&mut self, node: NodeId, spec: NodeSpecId) -> Result<()>;

    fn remove(&mut self, id: NodeId) -> Result<()>;

    fn contains(&self, id: &NodeId) -> bool;

    fn spec_id(&self, id: &NodeId) -> Option<&NodeSpecId>;
    fn nodes(&self) -> impl Iterator<Item = &NodeId>;
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

    fn nodes(&self) -> impl Iterator<Item = &NodeId> {
        self.nodes.keys()
    }
}

pub trait PortStateRepositoryConcept: Default {
    fn create(&mut self, node: NodeId, port: NodePortId, conn: NodePortConnection) -> Result<()>;
    fn remove(&mut self, node: NodeId, port: NodePortId) -> Option<NodePortConnection>;

    fn state(&self, node: NodeId, port: NodePortId) -> Option<&NodePortConnection>;
    fn state_mut(&mut self, node: NodeId, port: NodePortId) -> Option<&mut NodePortConnection>;
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
}

pub trait NodePositionRepositoryConcept: Default {
    fn update(&mut self, id: NodeId, position: NodePosition) -> Result<()>;
    fn position(&self, id: NodeId) -> Option<&NodePosition>;
    fn remove(&mut self, id: NodeId) -> Result<()>;
}

#[derive(Default)]
pub struct NodePositionRepository {
    positions: HashMap<NodeId, NodePosition>,
}

impl NodePositionRepositoryConcept for NodePositionRepository {
    fn update(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        self.positions.insert(id, position);
        Ok(())
    }

    fn remove(&mut self, id: NodeId) -> Result<()> {
        self.positions.remove(&id);
        Ok(())
    }

    fn position(&self, id: NodeId) -> Option<&NodePosition> {
        self.positions.get(&id)
    }
}

/// caller (service layer) has to assure that params are valid w.r.t. schema
pub trait ParamValuesRepositoryConcept: Default {
    fn insert(&mut self, id: NodeId, params: Parameters);
    fn remove(&mut self, id: NodeId) -> Result<()>;
    fn update(&mut self, id: NodeId, field_name: &str, value: Value) -> Result<()>;
    fn params(&self, id: NodeId) -> Option<&Parameters>;
}

#[derive(Default)]
pub struct ParamValuesRepository {
    params: HashMap<NodeId, Parameters>,
}

impl ParamValuesRepositoryConcept for ParamValuesRepository {
    fn insert(&mut self, id: NodeId, params: Parameters) {
        self.params.insert(id, params);
    }

    fn remove(&mut self, id: NodeId) -> Result<()> {
        self.params.remove(&id);
        Ok(())
    }

    fn update(&mut self, id: NodeId, field_name: &str, value: Value) -> Result<()> {
        self.params
            .get_mut(&id)
            .ok_or_else(|| anyhow!("no params for node id {id} have been registered"))?
            .update(field_name, value)
    }

    fn params(&self, id: NodeId) -> Option<&Parameters> {
        self.params.get(&id)
    }
}

pub trait EdgeRepositoryConcept: Default {
    fn create(&mut self, edge: NodeEdge) -> Result<EdgeId>;
    fn load(&mut self, id: EdgeId, edge: NodeEdge) -> Result<()>;

    fn remove(&mut self, id: EdgeId) -> Option<NodeEdge>;

    fn edges(&self) -> impl Iterator<Item = &NodeEdge>;
    fn ids(&self) -> impl Iterator<Item = EdgeId>;

    // provided methods
    fn iter(&self) -> impl Iterator<Item = (EdgeId, &NodeEdge)> {
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

    fn load(&mut self, id: EdgeId, edge: NodeEdge) -> Result<()> {
        if self.edges.contains_key(&id) {
            bail!("cannot load edge {id} as there is already an edge stored with the same id");
        }
        self.edges.insert(id, edge);
        Ok(())
    }

    fn remove(&mut self, id: EdgeId) -> Option<NodeEdge> {
        self.edges.remove(&id)
    }

    fn edges(&self) -> impl Iterator<Item = &NodeEdge> {
        self.edges.values()
    }

    fn ids(&self) -> impl Iterator<Item = EdgeId> {
        self.edges.keys().copied()
    }
}

pub trait ChannelDataRepositoryConcept: Default {
    fn create(&mut self, data: ChannelData) -> Result<ChannelId>;
    fn load(&mut self, node: ChannelId, data: ChannelData) -> Result<()>;

    fn remove(&mut self, id: ChannelId) -> Option<ChannelData>;

    fn contains(&self, id: &ChannelId) -> bool;

    fn data_mut(&mut self, id: ChannelId) -> Option<&mut ChannelData>;
    fn data(&self, id: ChannelId) -> Option<&ChannelData>;
    fn data_iter(&self) -> impl Iterator<Item = &ChannelData>;

    fn channels(&self) -> impl Iterator<Item = &ChannelId>;
}

pub struct ChannelDataInput {
    pub config: ChannelConfig,
    pub position: ChannelPosition,
}

pub struct ChannelData {
    pub spec_id: ChannelSpecId,
    pub config: ChannelConfig,
    pub position: ChannelPosition,
}

impl ChannelData {
    pub fn new(spec_id: ChannelSpecId, input: ChannelDataInput) -> Self {
        Self {
            spec_id,
            config: input.config,
            position: input.position,
        }
    }
}

#[derive(Default)]
pub struct ChannelDataRepository {
    channels: HashMap<ChannelId, ChannelData>,
    id_provider: IdProvider<ChannelId>,
}

impl ChannelDataRepositoryConcept for ChannelDataRepository {
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

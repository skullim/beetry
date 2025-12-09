use std::collections::{HashMap, HashSet};

use crate::{
    domain::models::{
        ChannelId, ChannelPosition, EdgeId, NodeChannelPortId, NodeEdge, NodePortConnection,
        NodePortSpec, NodeSpec, NodeSpecId,
    },
    id::IdProvider,
};

use super::models::{NodeId, NodePosition};
use beetry_serde::{
    de::{channel::ChannelParameters, parameter::Parameters},
    ser::channel::ChannelSpec,
};

use anyhow::{Result, anyhow, bail};
use derive_more::From;
use getset::{Getters, MutGetters};
use serde_value::Value;

#[derive(Debug, Default, Getters, MutGetters)]
pub struct EditorRepository<NRF, ER, CR> {
    #[getset(get = "pub", get_mut = "pub")]
    node: NRF,
    #[getset(get = "pub", get_mut = "pub")]
    edge: ER,
    #[getset(get = "pub", get_mut = "pub")]
    channel: CR,
}

pub struct EditorRepositoryView<'a, NRF, ER, CR> {
    pub node: &'a NRF,
    pub edge: &'a ER,
    pub channel: &'a CR,
}

pub struct EditorRepositoryViewMut<'a, NRF, ER, CR> {
    pub node: &'a mut NRF,
    pub edge: &'a mut ER,
    pub channel: &'a mut CR,
}

impl<NRF, ER, CR> EditorRepository<NRF, ER, CR> {
    pub fn view(&self) -> EditorRepositoryView<'_, NRF, ER, CR> {
        EditorRepositoryView {
            node: &self.node,
            edge: &self.edge,
            channel: &self.channel,
        }
    }

    pub fn view_mut(&mut self) -> EditorRepositoryViewMut<'_, NRF, ER, CR> {
        EditorRepositoryViewMut {
            node: &mut self.node,
            edge: &mut self.edge,
            channel: &mut self.channel,
        }
    }
}

impl<NRF, ER, CR> EditorRepository<NRF, ER, CR>
where
    NRF: Default,
    ER: Default,
    CR: Default,
{
    /// Instance should be initialized in default state, the interaction with concrete repositories
    /// should be managed by service layer.
    /// This saves ton of validation (to guarantee repositories are in correct state) that would be
    /// necessary to perform if Self would take the repositories in the constructor.
    pub fn new() -> Self {
        Self {
            node: NRF::default(),
            edge: ER::default(),
            channel: CR::default(),
        }
    }
}

pub trait NodeRepositoryConcept: Default {
    fn create(&mut self, spec: NodeSpecId) -> Result<NodeId>;
    fn remove(&mut self, id: NodeId) -> Result<()>;

    fn contains(&self, id: &NodeId) -> bool;

    fn spec_id(&self, id: &NodeId) -> Option<&NodeSpecId>;
    fn nodes(&self) -> NodeIter<'_>;
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

    fn nodes(&self) -> NodeIter<'_> {
        NodeIter::HashMap(self.nodes.keys())
    }
}

#[derive(Debug, From)]
pub(crate) enum NodeIter<'a> {
    Slice(std::slice::Iter<'a, NodeId>),
    HashSet(std::collections::hash_set::Iter<'a, NodeId>),
    HashMap(std::collections::hash_map::Keys<'a, NodeId, NodeSpecId>),
    Option(std::option::Iter<'a, NodeId>),
}

impl<'a> Iterator for NodeIter<'a> {
    type Item = &'a NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Slice(slice) => slice.next(),
            Self::HashSet(set) => set.next(),
            Self::HashMap(map) => map.next(),
            Self::Option(o) => o.next(),
        }
    }
}

/// Service layer should guarantee that no sam NodeSpecs are stored in repository
pub trait NodeSpecRepositoryConcept: Default {
    fn create(&mut self, spec: NodeSpec) -> Result<NodeSpecId>;
    /// Return error when id already used
    fn load(&mut self, id: NodeSpecId, spec: NodeSpec) -> Result<()>;

    fn remove(&mut self, id: NodeSpecId) -> Option<NodeSpec>;

    fn spec(&self, id: NodeSpecId) -> Option<&NodeSpec>;
}

#[derive(Debug, Default)]
pub struct NodeSpecRepository {
    specs: HashMap<NodeSpecId, NodeSpec>,
    id_provider: IdProvider<NodeSpecId>,
}

impl NodeSpecRepositoryConcept for NodeSpecRepository {
    fn create(&mut self, spec: NodeSpec) -> Result<NodeSpecId> {
        let id = self
            .id_provider
            .next_available_id(|id| !self.specs.contains_key(id));
        self.specs.insert(id, spec);
        Ok(id)
    }

    fn load(&mut self, id: NodeSpecId, spec: NodeSpec) -> Result<()> {
        if self.specs.contains_key(&id) {
            bail!("cannot load spec {id} as there is already spec stored with the same id");
        }
        self.specs.insert(id, spec);
        Ok(())
    }

    fn remove(&mut self, id: NodeSpecId) -> Option<NodeSpec> {
        self.specs.remove(&id)
    }

    fn spec(&self, id: NodeSpecId) -> Option<&NodeSpec> {
        self.specs.get(&id)
    }
}

pub trait NodeRepositoryFacadeConcept: Default {
    type NodeRepo: NodeRepositoryConcept;
    type SpecRepo: NodeSpecRepositoryConcept;

    type PositionRepo: NodePositionRepositoryConcept;
    type ParamRepo: ParamValuesRepositoryConcept;
    type PortRepo: NodePortRepositoryConcept;

    fn view(&self) -> NodeRepositoryFacadeView<'_, Self>
    where
        Self: Sized;

    fn view_mut(&mut self) -> NodeRepositoryFacadeViewMut<'_, Self>
    where
        Self: Sized;
}

pub trait NodePortRepositoryConcept: Default {
    fn create(&mut self, id: NodeId, port_specs: impl Iterator<Item = NodePortSpec>) -> Result<()>;

    fn spec(&self, node: NodeId, port: NodeChannelPortId) -> Option<&NodePortSpec>;

    fn connection(&self, node: NodeId, port: NodeChannelPortId) -> Option<&NodePortConnection>;
    fn connection_mut(
        &mut self,
        node: NodeId,
        port: NodeChannelPortId,
    ) -> Option<&mut NodePortConnection>;

    fn set_conn(
        &mut self,
        node: NodeId,
        port: NodeChannelPortId,
        kind: NodePortConnection,
    ) -> Result<()>;

    fn ports(&self, id: NodeId) -> impl Iterator<Item = NodeChannelPortId>;
}

#[derive(Default)]
pub struct NodePortRepository {
    ports: HashMap<NodeId, HashSet<NodeChannelPortId>>,
    connections: HashMap<(NodeId, NodeChannelPortId), NodePortConnection>,
    specs: HashMap<(NodeId, NodeChannelPortId), NodePortSpec>,
}

impl NodePortRepositoryConcept for NodePortRepository {
    fn create(&mut self, id: NodeId, port_specs: impl Iterator<Item = NodePortSpec>) -> Result<()> {
        let port_specs: Vec<_> = port_specs.collect();

        let count = port_specs.len() as NodeChannelPortId;
        self.ports.insert(id, (0..count).collect());

        for (port_id, spec) in port_specs.into_iter().enumerate() {
            self.specs.insert((id, port_id as NodeChannelPortId), spec);
        }

        Ok(())
    }

    fn set_conn(
        &mut self,
        node: NodeId,
        port: NodeChannelPortId,
        kind: NodePortConnection,
    ) -> Result<()> {
        self.connections.insert((node, port), kind);
        Ok(())
    }

    fn spec(&self, node: NodeId, port: NodeChannelPortId) -> Option<&NodePortSpec> {
        self.specs.get(&(node, port))
    }

    fn connection(&self, node: NodeId, port: NodeChannelPortId) -> Option<&NodePortConnection> {
        self.connections.get(&(node, port))
    }

    fn connection_mut(
        &mut self,
        node: NodeId,
        port: NodeChannelPortId,
    ) -> Option<&mut NodePortConnection> {
        self.connections.get_mut(&(node, port))
    }

    fn ports(&self, id: NodeId) -> impl Iterator<Item = NodeChannelPortId> {
        self.ports
            .get(&id)
            .into_iter()
            .flat_map(|set| set.iter().copied())
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

pub struct NodeRepositoryFacadeView<'a, F: NodeRepositoryFacadeConcept> {
    pub nodes: &'a F::NodeRepo,
    pub specs: &'a F::SpecRepo,

    pub positions: &'a F::PositionRepo,
    pub parameters: &'a F::ParamRepo,
    pub ports: &'a F::PortRepo,
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
    pub parameters: &'a mut F::ParamRepo,
    pub ports: &'a mut F::PortRepo,
}

#[derive(Default, Getters, MutGetters)]
pub struct NodeRepositoryFacade {
    node: NodeRepository,
    spec: NodeSpecRepository,

    position: NodePositionRepository,
    parameter: ParamValuesRepository,
    port: NodePortRepository,
}

impl NodeRepositoryFacade {
    pub fn new(
        node: NodeRepository,
        spec: NodeSpecRepository,

        position: NodePositionRepository,
        parameter: ParamValuesRepository,
        port: NodePortRepository,
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
    type ParamRepo = ParamValuesRepository;
    type PortRepo = NodePortRepository;

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
    /// caller has to assure that params are valid w.r.t. schema
    fn insert(&mut self, id: NodeId, params: Parameters) {
        self.params.insert(id, params);
    }

    fn remove(&mut self, id: NodeId) -> Result<()> {
        self.params.remove(&id);
        Ok(())
    }

    /// caller has to assure that value is valid w.r.t. schema
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
    fn create(&mut self, id: EdgeId, edge: NodeEdge) -> Result<()>;
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
    /// Vec<NodeEdge> would also be sufficient, but frontend is rendered more efficiently
    /// if each element has a unique and *stable* id. In that sense frontend is intrusive, but otherwise it would be very costly to
    /// map the ids on any other layer
    edges: HashMap<EdgeId, NodeEdge>,
}

impl EdgeRepositoryConcept for EdgeRepository {
    fn create(&mut self, id: EdgeId, edge: NodeEdge) -> Result<()> {
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

pub trait ChannelRepositoryConcept: Default {
    fn create(&mut self, id: ChannelId, spec: ChannelSpec) -> Result<()>;
    fn remove(&mut self, id: ChannelId) -> Result<()>;

    fn spec(&self, id: ChannelId) -> Option<&ChannelSpec>;

    fn contains(&self, id: ChannelId) -> bool;

    fn set_parameters(&mut self, id: ChannelId, params: ChannelParameters) -> Result<()>;
    fn parameters_mut(&mut self, id: ChannelId) -> Option<&mut ChannelParameters>;
    fn parameters(&self, id: ChannelId) -> Option<&ChannelParameters>;

    fn update_position(&mut self, id: ChannelId, position: ChannelPosition) -> Result<()>;
    fn position(&self, id: ChannelId) -> Option<&ChannelPosition>;

    fn channels(&self) -> impl Iterator<Item = ChannelId>;
}

#[derive(Default)]
pub struct ChannelRepository {
    //@todo can optimize similar to how node specs are cached
    channels: HashMap<ChannelId, ChannelSpec>,
    parameters: HashMap<ChannelId, ChannelParameters>,
    positions: HashMap<ChannelId, ChannelPosition>,
}

impl ChannelRepositoryConcept for ChannelRepository {
    fn create(&mut self, id: ChannelId, spec: ChannelSpec) -> Result<()> {
        self.channels.insert(id, spec);
        Ok(())
    }

    fn remove(&mut self, id: ChannelId) -> Result<()> {
        self.channels.remove(&id);
        self.positions.remove(&id);
        Ok(())
    }

    fn spec(&self, id: ChannelId) -> Option<&ChannelSpec> {
        self.channels.get(&id)
    }

    fn contains(&self, id: ChannelId) -> bool {
        self.channels.contains_key(&id)
    }

    fn set_parameters(&mut self, id: ChannelId, metadata: ChannelParameters) -> Result<()> {
        self.parameters.insert(id, metadata);
        Ok(())
    }

    fn parameters_mut(&mut self, id: ChannelId) -> Option<&mut ChannelParameters> {
        self.parameters.get_mut(&id)
    }

    fn parameters(&self, id: ChannelId) -> Option<&ChannelParameters> {
        self.parameters.get(&id)
    }

    fn update_position(&mut self, id: ChannelId, position: ChannelPosition) -> Result<()> {
        if !self.channels.contains_key(&id) {
            bail!("attempted to update position of channel {id} which has not been registered");
        }
        self.positions.insert(id, position);
        Ok(())
    }

    fn position(&self, id: ChannelId) -> Option<&ChannelPosition> {
        self.positions.get(&id)
    }

    fn channels(&self) -> impl Iterator<Item = ChannelId> {
        self.channels.keys().copied()
    }
}

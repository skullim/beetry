use std::collections::{HashMap, HashSet};

use crate::domain::models::{
    ChannelId, ChannelPosition, EdgeId, NodeChannelPortId, NodeEdge, NodePortConnection,
    NodePortSpec,
};

use super::models::{NodeId, NodeKind, NodePosition};
use beetry_plugin::{ActionSpec, ConditionSpec};
use beetry_serde::{
    de::{channel::ChannelParameters, parameter::Parameters},
    ser::{
        channel::ChannelSpec,
        node::{ControlSpec, DecoratorSpec, NodeName, RootSpec},
    },
};

use anyhow::{Result, anyhow, bail};
use derive_more::From;
use getset::{Getters, MutGetters};
use serde_value::Value;
use slotmap::SlotMap;

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
    type Spec: Clone + ProvideNodeName;

    fn create(&mut self, id: NodeId, spec: &Self::Spec) -> Result<()>;
    fn remove(&mut self, id: NodeId) -> Result<()>;

    fn contains(&self, id: NodeId) -> bool;

    fn spec(&self, id: NodeId) -> Option<&Self::Spec>;

    fn nodes(&self) -> NodeIter<'_>;
}

pub trait ProvideNodeName {
    fn name(&self) -> &NodeName;
}

macro_rules! derive_provide_node_name {
    ($ty: ty) => {
        impl ProvideNodeName for $ty {
            fn name(&self) -> &NodeName {
                &self.name
            }
        }
    };
}
derive_provide_node_name!(RootSpec);
derive_provide_node_name!(ActionSpec);
derive_provide_node_name!(ControlSpec);
derive_provide_node_name!(DecoratorSpec);

#[derive(Debug, Default)]
pub struct RootNodeRepository {
    root: Option<NodeId>,
    spec: Option<RootSpec>,
}

impl NodeRepositoryConcept for RootNodeRepository {
    type Spec = RootSpec;

    fn create(&mut self, id: NodeId, spec: &Self::Spec) -> Result<()> {
        match self.root {
            Some(_) => bail!("attempted to register root node twice"),
            None => {
                self.spec = Some(spec.clone());
                self.root = Some(id);
                Ok(())
            }
        }
    }

    fn remove(&mut self, _id: NodeId) -> Result<()> {
        self.root.take();
        Ok(())
    }

    fn contains(&self, id: NodeId) -> bool {
        self.root == Some(id)
    }

    fn spec(&self, _id: NodeId) -> Option<&Self::Spec> {
        self.spec.as_ref()
    }

    fn nodes(&self) -> NodeIter<'_> {
        NodeIter::Option(self.root.iter())
    }
}

type NodeSchemaId = slotmap::DefaultKey;

#[derive(Debug)]
pub struct NodeRepository<S> {
    nodes: HashMap<NodeId, NodeSchemaId>,
    //@todo caching should probably be implemented by service layer?
    cached_schema_keys: HashMap<NodeName, NodeSchemaId>,
    specs: SlotMap<NodeSchemaId, S>,
}

// manual implementation needed as otherwise Default would only be implemented when S: Default, which is not needed in this case
impl<S> Default for NodeRepository<S> {
    fn default() -> Self {
        Self {
            nodes: <_>::default(),
            cached_schema_keys: <_>::default(),
            specs: <_>::default(),
        }
    }
}

impl<S> NodeRepository<S>
where
    S: Clone + ProvideNodeName,
{
    fn create_impl(&mut self, id: NodeId, spec: &S) -> Result<()> {
        let name = spec.name();
        let key = match self.cached_schema_keys.get(name) {
            Some(key) => *key,
            None => {
                let new_key = self.specs.insert(spec.clone());
                self.cached_schema_keys.insert(name.clone(), new_key);
                new_key
            }
        };
        self.nodes.insert(id, key);
        Ok(())
    }

    fn remove_impl(&mut self, id: NodeId) -> Result<()> {
        self.nodes.remove(&id);
        Ok(())
    }

    fn contains_impl(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    fn spec_impl(&self, id: NodeId) -> Option<&S> {
        let schema_key = self.nodes.get(&id)?;
        self.specs.get(*schema_key)
    }

    fn nodes_impl(&self) -> NodeIter<'_> {
        NodeIter::HashMapKeys(self.nodes.keys())
    }
}

pub type ActionNodeRepository = NodeRepository<ActionSpec>;
pub type ConditionNodeRepository = NodeRepository<ConditionSpec>;
pub type ControlNodeRepository = NodeRepository<ControlSpec>;
pub type DecoratorNodeRepository = NodeRepository<DecoratorSpec>;

#[derive(Debug, From)]
pub(crate) enum NodeIter<'a> {
    Slice(std::slice::Iter<'a, NodeId>),
    HashMapKeys(std::collections::hash_map::Keys<'a, NodeId, NodeSchemaId>),
    Option(std::option::Iter<'a, NodeId>),
}

impl<'a> Iterator for NodeIter<'a> {
    type Item = NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Slice(slice) => slice.next().copied(),
            Self::HashMapKeys(keys) => keys.next().copied(),
            Self::Option(o) => o.next().copied(),
        }
    }
}

//@todo replace by generic trait impl
macro_rules! derive_node_repository {
    ($ty: ty, $spec: ty) => {
        impl NodeRepositoryConcept for $ty {
            type Spec = $spec;

            fn create(&mut self, id: NodeId, spec: &Self::Spec) -> Result<()> {
                self.create_impl(id, spec)
            }
            fn remove(&mut self, id: NodeId) -> Result<()> {
                self.remove_impl(id)
            }

            fn contains(&self, id: NodeId) -> bool {
                self.contains_impl(id)
            }

            fn spec(&self, id: NodeId) -> Option<&Self::Spec> {
                self.spec_impl(id)
            }

            fn nodes(&self) -> NodeIter<'_> {
                self.nodes_impl()
            }
        }
    };
}

derive_node_repository! {ActionNodeRepository, ActionSpec}
derive_node_repository! {ControlNodeRepository, ControlSpec}
derive_node_repository! {DecoratorNodeRepository, DecoratorSpec}

pub trait NodeRepositoryFacadeConcept: Default {
    type RootRepo: NodeRepositoryConcept<Spec = RootSpec>;
    type ActionRepo: NodeRepositoryConcept<Spec = ActionSpec>;
    type ConditionRepo: NodeRepositoryConcept<Spec = ConditionSpec>;
    type ControlRepo: NodeRepositoryConcept<Spec = ControlSpec>;
    type DecoratorRepo: NodeRepositoryConcept<Spec = DecoratorSpec>;

    type KindRepo: NodeKindRepositoryConcept;
    type PositionRepo: NodePositionRepositoryConcept;
    type ParamRepo: ParamRepositoryConcept;
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

pub trait NodeKindRepositoryConcept: Default {
    fn insert(&mut self, id: NodeId, kind: NodeKind) -> Result<()>;
    fn remove(&mut self, id: NodeId) -> Result<()>;
    fn kind(&self, id: NodeId) -> Option<NodeKind>;
}

#[derive(Default)]
pub struct NodeKindRepository {
    kinds: HashMap<NodeId, NodeKind>,
}

impl NodeKindRepositoryConcept for NodeKindRepository {
    fn insert(&mut self, id: NodeId, kind: NodeKind) -> Result<()> {
        self.kinds.insert(id, kind);
        Ok(())
    }

    fn remove(&mut self, id: NodeId) -> Result<()> {
        self.kinds.remove(&id);
        Ok(())
    }

    fn kind(&self, id: NodeId) -> Option<NodeKind> {
        self.kinds.get(&id).copied()
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
    pub root: &'a F::RootRepo,
    pub action: &'a F::ActionRepo,
    pub condition: &'a F::ConditionRepo,
    pub control: &'a F::ControlRepo,
    pub decorator: &'a F::DecoratorRepo,

    pub kinds: &'a F::KindRepo,
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
    pub root: &'a mut F::RootRepo,
    pub action: &'a mut F::ActionRepo,
    pub condition: &'a mut F::ConditionRepo,
    pub control: &'a mut F::ControlRepo,
    pub decorator: &'a mut F::DecoratorRepo,

    pub kinds: &'a mut F::KindRepo,
    pub positions: &'a mut F::PositionRepo,
    pub parameters: &'a mut F::ParamRepo,
    pub ports: &'a mut F::PortRepo,
}

#[derive(Default, Getters, MutGetters)]
pub struct NodeRepositoryFacade {
    root: RootNodeRepository,
    action: ActionNodeRepository,
    condition: ConditionNodeRepository,
    control: ControlNodeRepository,
    decorator: DecoratorNodeRepository,

    kinds: NodeKindRepository,
    positions: NodePositionRepository,
    parameters: ParamRepository,
    ports: NodePortRepository,
}

impl NodeRepositoryFacade {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        root: RootNodeRepository,
        action: ActionNodeRepository,
        condition: ConditionNodeRepository,
        control: ControlNodeRepository,
        decorator: DecoratorNodeRepository,
        kinds: NodeKindRepository,
        positions: NodePositionRepository,
        parameters: ParamRepository,
        ports: NodePortRepository,
    ) -> Self {
        Self {
            root,
            action,
            condition,
            control,
            decorator,
            kinds,
            positions,
            parameters,
            ports,
        }
    }
}

impl NodeRepositoryFacadeConcept for NodeRepositoryFacade {
    type ActionRepo = ActionNodeRepository;
    type ConditionRepo = ConditionNodeRepository;
    type ControlRepo = ControlNodeRepository;
    type DecoratorRepo = DecoratorNodeRepository;
    type RootRepo = RootNodeRepository;
    type KindRepo = NodeKindRepository;
    type PositionRepo = NodePositionRepository;
    type ParamRepo = ParamRepository;
    type PortRepo = NodePortRepository;

    fn view(&self) -> NodeRepositoryFacadeView<'_, Self> {
        NodeRepositoryFacadeView {
            root: &self.root,
            action: &self.action,
            condition: &self.condition,
            control: &self.control,
            decorator: &self.decorator,
            kinds: &self.kinds,
            positions: &self.positions,
            parameters: &self.parameters,
            ports: &self.ports,
        }
    }

    fn view_mut(&mut self) -> NodeRepositoryFacadeViewMut<'_, Self> {
        NodeRepositoryFacadeViewMut {
            root: &mut self.root,
            action: &mut self.action,
            condition: &mut self.condition,
            control: &mut self.control,
            decorator: &mut self.decorator,
            kinds: &mut self.kinds,
            positions: &mut self.positions,
            parameters: &mut self.parameters,
            ports: &mut self.ports,
        }
    }
}

pub trait ParamRepositoryConcept: Default {
    fn insert(&mut self, id: NodeId, params: Parameters);
    fn remove(&mut self, id: NodeId) -> Result<()>;
    fn update(&mut self, id: NodeId, field_name: &str, value: Value) -> Result<()>;

    fn params(&self, id: NodeId) -> Option<&Parameters>;
}

#[derive(Default)]
pub struct ParamRepository {
    params: HashMap<NodeId, Parameters>,
}

impl ParamRepositoryConcept for ParamRepository {
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

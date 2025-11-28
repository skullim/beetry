use std::collections::{HashMap, HashSet};

use crate::domain::models::{
    ChannelId, ChannelPosition, EdgeId, EdgePosition, ExternalReceivers, ExternalSenders, NodeEdge,
};

use super::models::{NodeId, NodeKind, NodePosition};
use beetry_core::MessageHash;
use beetry_plugin::{ActionSpec, ConditionSpec};
use beetry_serde::{
    de::parameter::Parameters,
    ser::{
        channel::ChannelSpec,
        node::{ControlSpec, DecoratorSpec, NodeName, RootSpec},
    },
};

use anyhow::{Result, anyhow, bail};
use getset::{Getters, MutGetters};
use serde_value::Value;
use slotmap::SlotMap;

#[derive(Getters, MutGetters)]
pub struct EditorRepository<NRF, ER, CR, PR> {
    #[getset(get = "pub", get_mut = "pub")]
    node: NRF,
    #[getset(get = "pub", get_mut = "pub")]
    edge: ER,
    #[getset(get = "pub", get_mut = "pub")]
    channel: CR,
    #[getset(get = "pub", get_mut = "pub")]
    parameter: PR,
}

impl<NRF, ER, CR, PR> EditorRepository<NRF, ER, CR, PR> {
    pub fn new(node: NRF, edge: ER, channel: CR, parameter: PR) -> Self {
        Self {
            node,
            edge,
            channel,
            parameter,
        }
    }
}

pub trait NodeRepositoryConcept {
    type Spec: Clone + ProvideNodeName;

    fn create(&mut self, id: NodeId, spec: &Self::Spec) -> Result<()>;
    fn remove(&mut self, id: NodeId) -> Result<()>;

    fn contains(&self, id: NodeId) -> bool;

    fn spec(&self, id: NodeId) -> Option<&Self::Spec>;

    fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()>;
    fn position(&self, id: NodeId) -> Option<&NodePosition>;

    fn nodes(&self) -> impl Iterator<Item = NodeId>;
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

pub struct RootNodeRepository {
    node: Option<NodeId>,
    spec: Option<RootSpec>,
    position: Option<NodePosition>,
}

impl NodeRepositoryConcept for RootNodeRepository {
    type Spec = RootSpec;

    fn create(&mut self, id: NodeId, spec: &Self::Spec) -> Result<()> {
        match self.node {
            Some(_) => bail!("attempted to register root node twice"),
            None => {
                self.spec = Some(spec.clone());
                self.node = Some(id);
                Ok(())
            }
        }
    }

    fn remove(&mut self, _id: NodeId) -> Result<()> {
        self.node.take();
        self.position.take();
        Ok(())
    }

    fn contains(&self, id: NodeId) -> bool {
        self.node == Some(id)
    }

    fn spec(&self, _id: NodeId) -> Option<&Self::Spec> {
        self.spec.as_ref()
    }

    fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        if self.node != Some(id) {
            bail!("attempted to update root position for id: {id} that is not registered as root");
        }
        self.position = Some(position);
        Ok(())
    }

    fn position(&self, _id: NodeId) -> Option<&NodePosition> {
        self.position.as_ref()
    }

    fn nodes(&self) -> impl Iterator<Item = NodeId> {
        self.node.iter().copied()
    }
}

type NodeSchemaId = slotmap::DefaultKey;

pub struct NodeRepository<S> {
    nodes: HashMap<NodeId, NodeSchemaId>,
    cached_schema_keys: HashMap<NodeName, NodeSchemaId>,
    specs: SlotMap<NodeSchemaId, S>,
    positions: HashMap<NodeId, NodePosition>,
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
        self.positions.remove(&id);
        Ok(())
    }

    fn contains_impl(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    fn spec_impl(&self, id: NodeId) -> Option<&S> {
        let schema_key = self.nodes.get(&id)?;
        self.specs.get(*schema_key)
    }

    fn update_position_impl(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        self.positions.insert(id, position);
        Ok(())
    }

    fn position_impl(&self, id: NodeId) -> Option<&NodePosition> {
        self.positions.get(&id)
    }

    fn nodes_impl(&self) -> impl Iterator<Item = NodeId> {
        self.nodes.keys().copied()
    }
}

pub type ActionNodeRepository = NodeRepository<ActionSpec>;
pub type ConditionNodeRepository = NodeRepository<ConditionSpec>;
pub type ControlNodeRepository = NodeRepository<ControlSpec>;
pub type DecoratorNodeRepository = NodeRepository<DecoratorSpec>;

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

            fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
                self.update_position_impl(id, position)
            }
            fn position(&self, id: NodeId) -> Option<&NodePosition> {
                self.position_impl(id)
            }

            fn nodes(&self) -> impl Iterator<Item = NodeId> {
                self.nodes_impl()
            }
        }
    };
}

derive_node_repository! {ActionNodeRepository, ActionSpec}
derive_node_repository! {ControlNodeRepository, ControlSpec}
derive_node_repository! {DecoratorNodeRepository, DecoratorSpec}

pub trait NodeRepositoryFacadeConcept {
    fn action(&self) -> &impl NodeRepositoryConcept<Spec = ActionSpec>;
    fn action_mut(&mut self) -> &mut impl NodeRepositoryConcept<Spec = ActionSpec>;

    fn condition(&self) -> &impl NodeRepositoryConcept<Spec = ConditionSpec>;
    fn condition_mut(&mut self) -> &mut impl NodeRepositoryConcept<Spec = ConditionSpec>;

    fn control(&self) -> &impl NodeRepositoryConcept<Spec = ControlSpec>;
    fn control_mut(&mut self) -> &mut impl NodeRepositoryConcept<Spec = ControlSpec>;

    fn decorator(&self) -> &impl NodeRepositoryConcept<Spec = DecoratorSpec>;
    fn decorator_mut(&mut self) -> &mut impl NodeRepositoryConcept<Spec = DecoratorSpec>;

    fn root(&self) -> &impl NodeRepositoryConcept<Spec = RootSpec>;
    fn root_mut(&mut self) -> &mut impl NodeRepositoryConcept<Spec = RootSpec>;

    fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()>;
    fn position(&self, id: NodeId) -> Option<&NodePosition>;
    fn remove_position(&mut self, id: NodeId) -> Result<()>;

    fn insert_kind(&mut self, id: NodeId, kind: NodeKind) -> Result<()>;
    fn kind(&self, id: NodeId) -> Option<NodeKind>;
    fn remove_kind(&mut self, id: NodeId) -> Result<()>;
}

#[derive(Getters, MutGetters)]
pub struct NodeRepositoryFacade {
    root: RootNodeRepository,
    action: ActionNodeRepository,
    condition: ConditionNodeRepository,
    control: ControlNodeRepository,
    decorator: DecoratorNodeRepository,

    kinds: HashMap<NodeId, NodeKind>,
    positions: HashMap<NodeId, NodePosition>,
}

impl NodeRepositoryFacadeConcept for NodeRepositoryFacade {
    fn action(&self) -> &impl NodeRepositoryConcept<Spec = ActionSpec> {
        &self.action
    }
    fn action_mut(&mut self) -> &mut impl NodeRepositoryConcept<Spec = ActionSpec> {
        &mut self.action
    }

    fn condition(&self) -> &impl NodeRepositoryConcept<Spec = ConditionSpec> {
        &self.condition
    }
    fn condition_mut(&mut self) -> &mut impl NodeRepositoryConcept<Spec = ConditionSpec> {
        &mut self.condition
    }

    fn control(&self) -> &impl NodeRepositoryConcept<Spec = ControlSpec> {
        &self.control
    }
    fn control_mut(&mut self) -> &mut impl NodeRepositoryConcept<Spec = ControlSpec> {
        &mut self.control
    }

    fn decorator(&self) -> &impl NodeRepositoryConcept<Spec = DecoratorSpec> {
        &self.decorator
    }
    fn decorator_mut(&mut self) -> &mut impl NodeRepositoryConcept<Spec = DecoratorSpec> {
        &mut self.decorator
    }

    fn root(&self) -> &impl NodeRepositoryConcept<Spec = RootSpec> {
        &self.root
    }
    fn root_mut(&mut self) -> &mut impl NodeRepositoryConcept<Spec = RootSpec> {
        &mut self.root
    }

    fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        self.positions.insert(id, position);
        Ok(())
    }

    fn position(&self, id: NodeId) -> Option<&NodePosition> {
        self.positions.get(&id)
    }

    fn remove_position(&mut self, id: NodeId) -> Result<()> {
        self.positions.remove(&id);
        Ok(())
    }

    fn insert_kind(&mut self, id: NodeId, kind: NodeKind) -> Result<()> {
        self.kinds.insert(id, kind);
        Ok(())
    }

    fn kind(&self, id: NodeId) -> Option<NodeKind> {
        self.kinds.get(&id).copied()
    }

    fn remove_kind(&mut self, id: NodeId) -> Result<()> {
        self.kinds.remove(&id);
        Ok(())
    }
}

pub trait ParamRepositoryConcept {
    fn insert(&mut self, id: NodeId, params: Parameters);
    fn remove(&mut self, id: NodeId) -> Result<()>;
    fn update(&mut self, id: NodeId, field_name: &str, value: Value) -> Result<()>;

    fn params(&self, id: NodeId) -> Option<&Parameters>;
}

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

pub trait EdgeRepositoryConcept {
    fn create(&mut self, id: EdgeId, edge: NodeEdge) -> Result<()>;
    fn remove(&mut self, id: EdgeId) -> Result<()>;

    fn update_position(&mut self, id: ChannelId, position: EdgePosition) -> Result<()>;
    fn position(&self, id: ChannelId) -> Option<&EdgePosition>;

    fn children_of(&self, id: NodeId) -> impl Iterator<Item = NodeId>;

    fn edge(&self, id: EdgeId) -> Option<&NodeEdge>;
    fn edges(&self) -> impl Iterator<Item = EdgeId>;

    fn on_node_removal(&mut self, id: NodeId) -> Result<()>;
}

pub struct EdgeRepository {}

// pub trait NodeKindRepositoryConcept {
//     fn insert(&mut self, id: NodeId, kind: NodeKind) -> Result<()>;
//     fn remove(&mut self, id: NodeId) -> Result<()>;

//     fn kind(&self, id: NodeId) -> Option<NodeKind>;
// }

// pub struct NodeKindRepository {
//     kinds: HashMap<NodeId, NodeKind>,
// }

// impl NodeKindRepositoryConcept for NodeKindRepository {
//     fn insert(&mut self, id: NodeId, kind: NodeKind) -> Result<()> {
//         self.kinds.insert(id, kind);
//         Ok(())
//     }

//     fn remove(&mut self, id: NodeId) -> Result<()> {
//         self.kinds.remove(&id);
//         Ok(())
//     }

//     fn kind(&self, id: NodeId) -> Option<NodeKind> {
//         self.kinds.get(&id).copied()
//     }
// }

pub trait ChannelRepositoryConcept {
    fn create(&mut self, id: ChannelId, spec: ChannelSpec) -> Result<()>;
    fn remove(&mut self, id: ChannelId) -> Result<()>;

    fn contains(&self, id: NodeId) -> bool;

    fn insert_sender(&mut self, id: ChannelId, from: NodeId);
    fn senders(&self) -> impl Iterator<Item = (NodeId, impl Iterator<Item = ChannelId>)>;

    fn insert_receiver(&mut self, id: ChannelId, to: NodeId);
    fn receivers(&self) -> impl Iterator<Item = (NodeId, impl Iterator<Item = ChannelId>)>;

    fn insert_external_sender(&mut self, id: NodeId, sender: MessageHash);
    fn external_senders(&self, id: NodeId) -> Option<&ExternalSenders>;

    fn insert_external_receiver(&mut self, id: NodeId, receiver: MessageHash);
    fn external_receivers(&self, id: NodeId) -> Option<&ExternalReceivers>;

    fn update_position(&mut self, id: ChannelId, position: ChannelPosition) -> Result<()>;
    fn position(&self, id: ChannelId) -> Option<&ChannelPosition>;

    fn channels(&self) -> impl Iterator<Item = ChannelId>;

    fn on_node_removal(&mut self, id: NodeId) -> Result<()>;
}

pub struct ChannelRepository {
    //@todo can optimize similar to how node specs are cached
    channels: HashMap<ChannelId, ChannelSpec>,
    senders: HashMap<NodeId, HashSet<ChannelId>>,
    receivers: HashMap<NodeId, HashSet<ChannelId>>,
    external_senders: HashMap<NodeId, HashSet<MessageHash>>,
    external_receivers: HashMap<NodeId, HashSet<MessageHash>>,
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

    fn contains(&self, id: NodeId) -> bool {
        self.channels.contains_key(&id)
    }

    fn insert_sender(&mut self, from: NodeId, id: ChannelId) {
        self.senders
            .entry(from)
            .and_modify(|senders| {
                senders.insert(id);
            })
            .or_insert_with(|| {
                let mut senders = HashSet::new();
                senders.insert(id);
                senders
            });
    }
    fn senders(&self) -> impl Iterator<Item = (NodeId, impl Iterator<Item = ChannelId>)> {
        self.senders
            .iter()
            .map(|(id, set)| (*id, set.iter().copied()))
    }

    fn insert_receiver(&mut self, to: NodeId, id: ChannelId) {
        self.receivers
            .entry(to)
            .and_modify(|receivers| {
                receivers.insert(id);
            })
            .or_insert_with(|| {
                let mut receivers = HashSet::new();
                receivers.insert(id);
                receivers
            });
    }
    fn receivers(&self) -> impl Iterator<Item = (NodeId, impl Iterator<Item = ChannelId>)> {
        self.receivers
            .iter()
            .map(|(id, set)| (*id, set.iter().copied()))
    }

    fn insert_external_sender(&mut self, id: NodeId, sender: MessageHash) {
        self.external_senders
            .entry(id)
            .and_modify(|senders| {
                senders.insert(sender);
            })
            .or_insert_with(|| {
                let mut senders = HashSet::new();
                senders.insert(sender);
                senders
            });
    }
    fn external_senders(&self, id: NodeId) -> Option<&ExternalSenders> {
        todo!()
    }

    fn insert_external_receiver(&mut self, id: NodeId, receiver: MessageHash) {
        self.external_receivers
            .entry(id)
            .and_modify(|receivers| {
                receivers.insert(receiver);
            })
            .or_insert_with(|| {
                let mut receivers = HashSet::new();
                receivers.insert(receiver);
                receivers
            });
    }
    fn external_receivers(&self, id: NodeId) -> Option<&ExternalReceivers> {
        todo!()
    }

    fn update_position(&mut self, id: ChannelId, position: ChannelPosition) -> Result<()> {
        if self.channels.contains_key(&id) {
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

    fn on_node_removal(&mut self, id: NodeId) -> Result<()> {
        self.receivers.remove(&id);
        self.senders.remove(&id);
        self.external_receivers.remove(&id);
        self.external_senders.remove(&id);
        Ok(())
    }
}

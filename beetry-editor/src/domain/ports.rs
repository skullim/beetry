use std::collections::{HashMap, HashSet};

use crate::domain::models::{ChannelId, EdgeId, ExternalReceivers, ExternalSenders, NodeEdge};

use super::models::{NodeId, NodeKind, NodePosition};
use beetry_core::MessageHash;
use beetry_plugin::{ActionSpec, ConditionSpec};
use beetry_serde::{
    de::parameter::Parameters,
    ser::node::{ControlSpec, DecoratorSpec, NodeName},
};

use anyhow::{Result, anyhow, bail};
use serde_value::Value;
use slotmap::SlotMap;

pub trait NodeRepository {
    fn create_root(&mut self, id: NodeId) -> Result<()>;
    fn create_action(&mut self, id: NodeId) -> Result<()>;
    fn create_condition(&mut self, id: NodeId) -> Result<()>;
    fn create_control(&mut self, id: NodeId) -> Result<()>;
    fn create_decorator(&mut self, id: NodeId) -> Result<()>;

    fn remove(&mut self, id: NodeId) -> Result<()>;

    fn kind(&self, id: NodeId) -> Option<NodeKind>;

    fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()>;

    fn position(&self, id: NodeId) -> Option<&NodePosition>;

    fn nodes(&self) -> impl Iterator<Item = NodeId>;
}

pub trait NodeRepositoryConcept {
    type Spec: Clone + ProvideNodeName;

    fn create(&mut self, spec: &Self::Spec, id: NodeId) -> Result<()>;
    fn remove(&mut self, id: NodeId) -> Result<()>;

    fn register_spec(&mut self, spec: Self::Spec) -> Result<()>;
    fn spec(&self, id: NodeId) -> Option<&Self::Spec>;

    fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()>;
    fn position(&self, id: NodeId) -> Option<&NodePosition>;

    fn nodes(&self) -> impl Iterator<Item = NodeId>;
}

type NodeSchemaId = slotmap::DefaultKey;

pub struct NodeRepositoryProto<S> {
    nodes: HashMap<NodeId, NodeSchemaId>,
    cached_schema_keys: HashMap<NodeName, NodeSchemaId>,
    specs: SlotMap<NodeSchemaId, S>,
    positions: HashMap<NodeId, NodePosition>,
}

impl<S> NodeRepositoryProto<S>
where
    S: Clone + ProvideNodeName,
{
    fn create_impl(&mut self, spec: &S, id: NodeId) -> Result<()> {
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

    fn register_spec_impl(&mut self, spec: S) -> Result<()> {
        let name = spec.name().clone();
        let key = self.specs.insert(spec);
        self.cached_schema_keys.insert(name, key);
        Ok(())
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

pub type ActionNodeRepository = NodeRepositoryProto<ActionSpec>;
pub type ConditionNodeRepository = NodeRepositoryProto<ConditionSpec>;
pub type ControlNodeRepository = NodeRepositoryProto<ControlSpec>;
pub type DecoratorNodeRepository = NodeRepositoryProto<DecoratorSpec>;

//@todo replace by generic trait impl
macro_rules! derive_node_repository {
    ($ty: ty, $spec: ty) => {
        impl NodeRepositoryConcept for $ty {
            type Spec = $spec;

            fn create(&mut self, spec: &Self::Spec, id: NodeId) -> Result<()> {
                self.create_impl(spec, id)
            }
            fn remove(&mut self, id: NodeId) -> Result<()> {
                self.remove_impl(id)
            }

            fn register_spec(&mut self, spec: Self::Spec) -> Result<()> {
                self.register_spec_impl(spec)
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

derive_provide_node_name!(ActionSpec);
derive_provide_node_name!(ControlSpec);
derive_provide_node_name!(DecoratorSpec);

derive_node_repository! {ActionNodeRepository, ActionSpec}
derive_node_repository! {ControlNodeRepository, ControlSpec}
derive_node_repository! {DecoratorNodeRepository, DecoratorSpec}

pub trait SpecRepository {
    fn register_action(&mut self, spec: ActionSpec) -> Result<()>;
    fn register_condition(&mut self, spec: ConditionSpec) -> Result<()>;
    fn register_control(&mut self, spec: ControlSpec) -> Result<()>;
    fn register_decorator(&mut self, spec: DecoratorSpec) -> Result<()>;

    fn bind_action_id(&mut self, name: &NodeName, id: NodeId) -> Result<()>;
    fn bind_condition_id(&mut self, name: &NodeName, id: NodeId) -> Result<()>;
    fn bind_control_id(&mut self, name: &NodeName, id: NodeId) -> Result<()>;
    fn bind_decorator_id(&mut self, name: &NodeName, id: NodeId) -> Result<()>;

    fn action(&self, id: NodeId) -> Option<&ActionSpec>;
    fn condition(&self, id: NodeId) -> Option<&ConditionSpec>;
    fn control(&self, id: NodeId) -> Option<&ControlSpec>;
    fn decorator(&self, id: NodeId) -> Option<&DecoratorSpec>;
}

pub trait ParamRepository {
    fn insert(&mut self, id: NodeId, params: Parameters);
    fn update(&mut self, id: NodeId, field_name: &str, value: Value) -> Result<()>;
    fn params(&self, id: NodeId) -> &Parameters;
}

pub trait ExternalPortRepository {
    fn insert_sender(&mut self, id: NodeId, sender: MessageHash);
    fn senders(&self, id: NodeId) -> &ExternalSenders;

    fn insert_receiver(&mut self, id: NodeId, receiver: MessageHash);
    fn receivers(&self, id: NodeId) -> &ExternalReceivers;
}

pub trait EdgeRepository {
    fn create(&mut self, edge: NodeEdge) -> Result<EdgeId>;
    fn remove(&mut self, id: EdgeId) -> Result<()>;
    fn edges(&self) -> impl Iterator<Item = EdgeId>;
}

pub trait ChannelRepository {
    fn create(&mut self) -> Result<ChannelId>;
    fn remove(&mut self, id: ChannelId) -> Result<()>;
    fn channels(&self) -> impl Iterator<Item = ChannelId>;
}

pub struct NodeRepositoryImpl {
    root: Option<NodeId>,
    actions: HashSet<NodeId>,
    conditions: HashSet<NodeId>,
    controls: HashSet<NodeId>,
    decorators: HashSet<NodeId>,
    positions: HashMap<NodeId, NodePosition>,
    kinds: HashMap<NodeId, NodeKind>,
}

impl NodeRepositoryImpl {
    fn unique_insert(set: &mut HashSet<NodeId>, id: NodeId) -> Result<()> {
        if set.insert(id) {
            bail!("attempted to insert node with {id} twice");
        }
        Ok(())
    }
}

impl NodeRepository for NodeRepositoryImpl {
    fn create_root(&mut self, id: NodeId) -> Result<()> {
        if self.root.is_some() {
            bail!("attempted to register root node with id {id} twice");
        }
        self.root = Some(id);
        Ok(())
    }

    fn create_action(&mut self, id: NodeId) -> Result<()> {
        Self::unique_insert(&mut self.actions, id)
    }

    fn create_condition(&mut self, id: NodeId) -> Result<()> {
        Self::unique_insert(&mut self.conditions, id)
    }

    fn create_control(&mut self, id: NodeId) -> Result<()> {
        Self::unique_insert(&mut self.controls, id)
    }

    fn create_decorator(&mut self, id: NodeId) -> Result<()> {
        Self::unique_insert(&mut self.decorators, id)
    }

    fn remove(&mut self, id: NodeId) -> Result<()> {
        for set in [
            &mut self.actions,
            &mut self.conditions,
            &mut self.controls,
            &mut self.decorators,
        ] {
            if set.remove(&id) {
                return Ok(());
            }
        }
        bail!("attempted to remove node {id} which was not stored in any node set");
    }

    fn kind(&self, id: NodeId) -> Option<NodeKind> {
        self.kinds.get(&id).copied()
    }

    fn nodes(&self) -> impl Iterator<Item = NodeId> {
        self.root
            .iter()
            .copied()
            .chain(self.actions.iter().copied())
            .chain(self.conditions.iter().copied())
            .chain(self.controls.iter().copied())
            .chain(self.decorators.iter().copied())
    }

    fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        self.positions.insert(id, position);
        Ok(())
    }

    fn position(&self, id: NodeId) -> Option<&NodePosition> {
        self.positions.get(&id)
    }
}

struct SpecWithBoundedNodes<T> {
    spec: T,
    nodes: HashSet<NodeId>,
}

impl<T> SpecWithBoundedNodes<T> {
    fn new(spec: T) -> Self {
        Self {
            spec,
            nodes: Default::default(),
        }
    }
}

pub struct SpecRepositoryImpl {
    //@todo probably better to split into id maps
    action_registry: HashMap<NodeName, SpecWithBoundedNodes<ActionSpec>>,
    condition_registry: HashMap<NodeName, SpecWithBoundedNodes<ConditionSpec>>,
    control_registry: HashMap<NodeName, SpecWithBoundedNodes<ControlSpec>>,
    decorator_registry: HashMap<NodeName, SpecWithBoundedNodes<DecoratorSpec>>,
}

// impl SpecRepositoryImpl {
//     fn unique_registration<S>(
//         map: &mut HashMap<NodeName, S>,
//         name: NodeName,
//         spec: S,
//     ) -> Result<()> {
//         if map.contains_key(&name) {
//             bail!("attempted to register node {name} twice");
//         }
//         map.insert(name, spec);
//         Ok(())
//     }

//     fn bind_node_id<S>(
//         registry: &mut HashMap<NodeName, SpecWithBoundedNodes<S>>,
//         name: &NodeName,
//         id: NodeId,
//     ) -> Result<()> {
//         registry
//             .get_mut(&name)
//             .ok_or_else(|| anyhow!("no node spec {name} registered"))?
//             .nodes
//             .insert(id);
//         Ok(())
//     }
// }

// impl SpecRepository for SpecRepositoryImpl {
//     fn register_action(&mut self, spec: ActionSpec) -> Result<()> {
//         Self::unique_registration(
//             &mut self.action_registry,
//             spec.name.clone(),
//             SpecWithBoundedNodes::new(spec),
//         )
//     }
//     fn register_condition(&mut self, spec: ConditionSpec) -> Result<()> {
//         Self::unique_registration(
//             &mut self.condition_registry,
//             spec.name.clone(),
//             SpecWithBoundedNodes::new(spec),
//         )
//     }
//     fn register_control(&mut self, spec: ControlSpec) -> Result<()> {
//         Self::unique_registration(
//             &mut self.control_registry,
//             spec.name.clone(),
//             SpecWithBoundedNodes::new(spec),
//         )
//     }
//     fn register_decorator(&mut self, spec: DecoratorSpec) -> Result<()> {
//         Self::unique_registration(
//             &mut self.decorator_registry,
//             spec.name.clone(),
//             SpecWithBoundedNodes::new(spec),
//         )
//     }

//     fn bind_action_id(&mut self, name: &NodeName, id: NodeId) -> Result<()> {
//         Self::bind_node_id(&mut self.action_registry, name, id)
//     }
//     fn bind_condition_id(&mut self, name: &NodeName, id: NodeId) -> Result<()> {
//         Self::bind_node_id(&mut self.condition_registry, name, id)
//     }
//     fn bind_control_id(&mut self, name: &NodeName, id: NodeId) -> Result<()> {
//         Self::bind_node_id(&mut self.control_registry, name, id)
//     }
//     fn bind_decorator_id(&mut self, name: &NodeName, id: NodeId) -> Result<()> {
//         Self::bind_node_id(&mut self.decorator_registry, name, id)
//     }

//     fn action(&self, id: NodeId) -> Option<&ActionSpec> {
//         self.action_registry.get(&id).map(|v| &v.spec)
//     }
//     fn condition(&self, id: NodeId) -> Option<&ConditionSpec>;
//     fn control(&self, id: NodeId) -> Option<&ControlSpec>;
//     fn decorator(&self, id: NodeId) -> Option<&DecoratorSpec>;
// }

use std::sync::Arc;

use crate::{id::NodePortId, spec::message::MessageSpec};
use anyhow::{Result, anyhow};
use bon::Builder;
use derive_more::{Display, From};
use getset::{CopyGetters, Getters, MutGetters};
use mitsein::{btree_map1::BTreeMap1, iter1::FromIterator1};
use serde::{Deserialize, Serialize};

#[derive(Debug, Builder, Clone, Getters, MutGetters)]
pub struct NodeSpec {
    #[getset(get = "pub")]
    pub key: NodeSpecKey,
    //@todo value
    #[getset(get = "pub")]
    params: Option<ParamsSpec>,
    #[getset(get = "pub", get_mut = "pub")]
    ports: Option<PortsSpec>,
}

impl NodeSpec {
    pub fn root() -> Self {
        NodeSpec::builder().key(NodeSpecKey::root()).build()
    }

    pub fn name(&self) -> &NodeName {
        self.key.name()
    }

    pub fn kind(&self) -> NodeKind {
        self.key.kind()
    }

    pub fn has_params(&self) -> bool {
        self.params.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters, CopyGetters, Serialize, Deserialize)]
pub struct NodeSpecKey {
    #[getset(get = "pub")]
    name: NodeName,
    #[getset(get_copy = "pub")]
    kind: NodeKind,
}

impl NodeSpecKey {
    pub fn new(name: NodeName, kind: NodeKind) -> Self {
        Self { name, kind }
    }

    pub fn root() -> Self {
        Self {
            name: NodeName::new("Root"),
            kind: NodeKind::Root,
        }
    }
}

#[derive(Debug, Default, Builder, Clone, Getters)]
pub struct NodeSpecValue {
    pub params: Option<ParamsSpec>,
    pub ports: Option<PortsSpec>,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, From,
)]
pub struct NodeName(pub String);

impl NodeName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl From<&'static str> for NodeName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, From, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    Control,
    Decorator,
    Leaf(LeafKind),
    Root,
}

impl NodeKind {
    pub fn action() -> Self {
        NodeKind::Leaf(LeafKind::Action)
    }

    pub fn condition() -> Self {
        NodeKind::Leaf(LeafKind::Condition)
    }

    pub fn leaf(&self) -> Option<LeafKind> {
        match self {
            NodeKind::Leaf(leaf) => Some(*leaf),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LeafKind {
    Action,
    Condition,
}

// Ports

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortsSpec {
    map: BTreeMap1<NodePortId, NodePortSpec>,
}

impl FromIterator1<NodePortSpec> for PortsSpec {
    fn from_iter1<I>(items: I) -> Self
    where
        I: mitsein::prelude::IntoIterator1<Item = NodePortSpec>,
    {
        Self {
            map: items
                .into_iter1()
                .enumerate()
                .map(|(id, spec)| (NodePortId::new(id as u8), spec))
                .collect1(),
        }
    }
}

impl PortsSpec {
    pub fn ids(&self) -> impl Iterator<Item = &NodePortId> {
        self.map.keys1().into_iter()
    }

    pub fn kind(&self, id: NodePortId) -> Option<NodePortKind> {
        self.map.get(&id).map(|spec| spec.kind)
    }

    pub fn sender_ids(&self) -> impl Iterator<Item = &NodePortId> {
        self.senders().map(|(id, _)| id)
    }

    pub fn receiver_ids(&self) -> impl Iterator<Item = &NodePortId> {
        self.receivers().map(|(id, _)| id)
    }

    pub fn senders(&self) -> impl Iterator<Item = (&NodePortId, &NodePortSpec)> {
        self.iter()
            .filter(|(_, spec)| spec.kind == NodePortKind::Sender)
    }

    pub fn receivers(&self) -> impl Iterator<Item = (&NodePortId, &NodePortSpec)> {
        self.iter()
            .filter(|(_, spec)| spec.kind == NodePortKind::Receiver)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&NodePortId, &NodePortSpec)> {
        self.ids().zip(self.specs())
    }

    pub fn spec(&self, id: NodePortId) -> Result<&NodePortSpec> {
        self.map
            .get(&id)
            .ok_or_else(|| anyhow!("failed to obtain spec for port {id}"))
    }

    pub fn specs(&self) -> impl Iterator<Item = &NodePortSpec> {
        self.map.values1().into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodePortSpec {
    pub kind: NodePortKind,
    pub msg_spec: MessageSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodePortKind {
    Sender,
    Receiver,
}

// Parameters

pub trait ProvideParamSpec {
    fn provide() -> ParamsSpec;
}

#[derive(Debug, Clone)]
pub struct ParamsSpec {
    map: BTreeMap1<FieldName, FieldDefinition>,
}

impl ParamsSpec {
    pub fn get(&self, name: &FieldName) -> Option<&FieldDefinition> {
        self.map.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&FieldName, &FieldDefinition)> {
        self.map.iter1().into_iter()
    }
}

impl FromIterator1<(FieldName, FieldDefinition)> for ParamsSpec {
    fn from_iter1<I>(items: I) -> Self
    where
        I: mitsein::prelude::IntoIterator1<Item = (FieldName, FieldDefinition)>,
    {
        Self {
            map: items.into_iter1().collect1(),
        }
    }
}

//@todo check if this or function pointer are better options
type SharedValidationFn<T> = Arc<dyn Fn(&T) -> Result<()>>;

#[derive(Default, Clone)]
pub struct FieldMetadata<T> {
    validation_fn: Option<SharedValidationFn<T>>,
}

impl<T> FieldMetadata<T> {
    pub fn new(validation_fn: SharedValidationFn<T>) -> Self {
        Self {
            validation_fn: Some(validation_fn),
        }
    }

    pub fn validate(&self, val: &T) -> Result<()> {
        self.validation_fn
            .as_ref()
            .map(|func| (func)(val))
            .unwrap_or(Ok(()))
    }
}

impl<T> std::fmt::Debug for FieldMetadata<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldMetadata")
            .field("validation_fn", &"{...}")
            .finish()
    }
}

pub type BoolFieldMetadata = FieldMetadata<bool>;
pub type U64FieldMetadata = FieldMetadata<u64>;
pub type I64FieldMetadata = FieldMetadata<i64>;
pub type F64FieldMetadata = FieldMetadata<f64>;
pub type StringFieldMetadata = FieldMetadata<String>;

#[derive(Debug, Clone)]
pub enum FieldTypeSpec {
    Bool(BoolFieldMetadata),
    U64(U64FieldMetadata),
    I64(I64FieldMetadata),
    F64(F64FieldMetadata),
    String(StringFieldMetadata),
}

pub type FieldName = String;

#[derive(Debug, Clone, Builder)]
pub struct FieldDefinition {
    pub type_spec: FieldTypeSpec,
    #[builder(into)]
    pub description: Option<String>,
}

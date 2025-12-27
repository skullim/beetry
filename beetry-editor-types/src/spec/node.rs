use crate::{id::NodePortId, spec::message::MessageSpec};
use anyhow::{Result, anyhow};
use bon::Builder;
use derive_more::{Display, From};
use getset::{CopyGetters, Getters};
use mitsein::{btree_map1::BTreeMap1, iter1::FromIterator1};
use serde::{Deserialize, Serialize};

#[derive(Debug, Builder, Clone, Getters)]
pub struct NodeSpec {
    #[getset(get = "pub")]
    pub key: NodeSpecKey,
    //@todo value
    #[getset(get = "pub")]
    params: Option<ParamsSpec>,
    #[getset(get = "pub")]
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParamsSpec {
    pub defs: Vec<Definition>,
}

impl ParamsSpec {
    pub fn new(defs: impl IntoIterator<Item = Definition>) -> Self {
        Self {
            defs: defs.into_iter().collect(),
        }
    }
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

pub trait ProvideSchema {
    fn provide() -> ParamsSpec;
}

#[derive(Debug, Clone, Builder, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Definition {
    #[builder(into)]
    pub name: String,
    pub ty: Type,
    #[builder(into)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Builder, PartialEq, Eq, Hash, Serialize, Deserialize, CopyGetters)]
#[get_copy = "pub"]
pub struct Bounds {
    min: i64,
    max: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    Boolean,
    Integer { bounds: Option<Bounds> },
    Float { bounds: Option<Bounds> },
    String { max_length: Option<usize> },
}

//@todo: Switch to Schema2

pub struct Schema2 {
    pub defs: Vec<Definition2>,
}

impl Schema2 {
    pub fn new(defs: impl IntoIterator<Item = Definition2>) -> Self {
        Self {
            defs: defs.into_iter().collect(),
        }
    }
}

type BoxValidationFn<T> = Box<dyn Fn(&T) -> Result<()>>;
type BoolValidationFn = BoxValidationFn<bool>;
type IntegerValidationFn = BoxValidationFn<i32>;
type FloatValidationFn = BoxValidationFn<f32>;
type StringValidationFn = BoxValidationFn<String>;

pub struct ValidationFns<T> {
    fns: Vec<BoxValidationFn<T>>,
}

impl<T> ValidationFns<T> {
    pub fn validate(&self, value: T) -> bool {
        self.fns.iter().all(|func| (func)(&value).is_err())
    }
}

pub enum Type2 {
    Boolean(BoolValidationFn),
    Integer(IntegerValidationFn),
    Float(FloatValidationFn),
    String(StringValidationFn),
}

//@todo probably need param id that is static regarding given node

#[derive(Builder)]
pub struct Definition2 {
    #[builder(into)]
    pub name: String,
    pub ty: Type2,
    #[builder(into)]
    pub description: Option<String>,
}

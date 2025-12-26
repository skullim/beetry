use crate::{id::NodePortId, spec::message::MessageSpec};
use anyhow::{Result, anyhow};
use bon::Builder;
use derive_more::{Display, From};
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

#[derive(Debug, Builder, Clone, Getters)]
pub struct NodeSpec {
    #[getset(get = "pub")]
    pub key: NodeSpecKey,
    //@todo value
    #[builder(default)]
    #[getset(get = "pub")]
    params: crate::spec::node::Schema,
    #[builder(default)]
    #[getset(get = "pub")]
    ports: PortsSpec,
}

impl NodeSpec {
    pub fn root() -> Self {
        NodeSpec::builder().key(NodeSpecKey::root()).build()
    }

    pub fn name(&self) -> &NodeName {
        &self.key.name
    }

    pub fn kind(&self) -> NodeKind {
        self.key.kind
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

#[derive(Debug, Default, Builder, Clone)]
pub struct NodeSpecValue {
    #[builder(default)]
    pub params: crate::spec::node::Schema,
    #[builder(default)]
    pub ports: PortsSpec,
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
pub struct Schema {
    pub defs: Vec<Definition>,
}

impl Schema {
    pub fn new(defs: impl IntoIterator<Item = Definition>) -> Self {
        Self {
            defs: defs.into_iter().collect(),
        }
    }
}

// Ports

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct PortsSpec {
    spec: Vec<NodePortSpec>,
    ids: Vec<NodePortId>,
}

impl PortsSpec {
    //@todo tedious to use, sometimes only senders or receivers are present
    // also order matters here, much better to define builder
    pub fn new(
        senders: impl IntoIterator<Item = MessageSpec>,
        receivers: impl IntoIterator<Item = MessageSpec>,
    ) -> Self {
        let spec: Vec<_> = senders
            .into_iter()
            .map(|spec| NodePortSpec {
                kind: NodePortKind::Sender,
                msg_spec: spec,
            })
            .chain(receivers.into_iter().map(|spec| NodePortSpec {
                kind: NodePortKind::Receiver,
                msg_spec: spec,
            }))
            .collect();
        let ids: Vec<_> = (0..spec.len())
            .map(|id| NodePortId::new(id.try_into().unwrap()))
            .collect();
        Self { spec, ids }
    }

    pub fn ids(&self) -> &[NodePortId] {
        &self.ids
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
        self.ids().iter().zip(self.specs())
    }

    pub fn spec(&self, id: NodePortId) -> Result<&NodePortSpec> {
        self.spec
            .get(id.raw_value() as usize)
            .ok_or_else(|| anyhow!("failed to obtain spec for port {id}"))
    }

    pub fn specs(&self) -> &[NodePortSpec] {
        &self.spec
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
    fn provide() -> Schema;
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

#[derive(Builder)]
pub struct Definition2 {
    #[builder(into)]
    pub name: String,
    pub ty: Type2,
    #[builder(into)]
    pub description: Option<String>,
}

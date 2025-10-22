mod channel;
mod leaf;
mod node;
mod root;
mod task;
mod tree;

pub use leaf::{Action, ActionBehavior, Condition, ConditionBehavior};
#[cfg(any(test, feature = "mock"))]
pub use node::MockNode;
pub use node::{BoxNode, Node};

pub use root::Root;
pub use tree::{Ticker as BehaviorTreeTicker, Tree, TreeEngine};

pub use task::{
    AbortTask, ExecutorConcept, NodeTask, NodeTaskFuture, QueryTask, RegisterTask, Task,
    TaskControl, TaskDescription, TaskStatus,
};

pub use channel::{BoxReceiver, BoxSender, Receiver, Sender, TryRecvResult, TrySendResult, error};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TickStatus {
    Success,
    Running,
    Failure,
}

impl TickStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Success | Self::Failure)
    }
}

use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schema {
    pub params: Vec<Definition>,
}

impl Schema {
    pub fn new(params: impl IntoIterator<Item = Definition>) -> Self {
        Self {
            params: params.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Builder, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    #[builder(into)]
    pub name: String,
    pub ty: Type,
    #[builder(into)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Builder, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct Bounds {
    #[getter(copy)]
    min: i32,
    #[getter(copy)]
    max: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    Boolean,
    Integer { bounds: Option<Bounds> },
    Float { bounds: Option<Bounds> },
    String { max_length: Option<usize> },
}

pub trait ProvideSchema {
    fn provide() -> Schema;
}

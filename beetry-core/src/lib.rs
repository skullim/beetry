mod leaf;
mod root;
mod task;
mod tree;

pub use leaf::{Action, ActionBehavior, Condition, ConditionBehavior};
pub use root::Root;
pub use task::{
    AbortTask, ExecutorConcept, NodeTask, NodeTaskFuture, QueryTask, RegisterTask, Task,
    TaskControl, TaskDescription, TaskStatus,
};
pub use tree::{BehaviorTree, Ticker as BehaviorTreeTicker, TreeEngine};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeStatus {
    Success,
    Running,
    Failure,
}

impl TreeStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Success | Self::Failure)
    }
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait Node {
    fn tick(&mut self) -> TreeStatus;
    /// reset given node to its default state:
    /// - should only be called by the behavior tree (root)
    /// - should not block the thread and ideally be finished during single tick
    fn reset(&mut self) {}

    /// interface to abort running tasks. The work is to be done by the leaf (action) nodes only, as control or decorator nodes are not scheduled on the executor
    fn abort(&mut self) {}
}

pub type BoxedNode = Box<dyn Node>;

impl Node for BoxedNode {
    fn tick(&mut self) -> TreeStatus {
        (**self).tick()
    }

    fn reset(&mut self) {
        (**self).reset()
    }

    fn abort(&mut self) {
        (**self).abort()
    }
}

use std::result::Result as StdResult;

pub type TryRecvResult<T> = StdResult<T, error::TryRecvError>;
pub type TrySendResult<T> = StdResult<(), error::TrySendError<T>>;

pub trait Receiver<T> {
    fn try_recv(&mut self) -> TryRecvResult<T>;
    fn drain(&mut self) {
        while self.try_recv().is_ok() {}
    }
}

pub type BoxedReceiver<T> = Box<dyn Receiver<T>>;

impl<T: 'static> Receiver<T> for BoxedReceiver<T> {
    fn try_recv(&mut self) -> TryRecvResult<T> {
        (**self).try_recv()
    }
}

pub trait Sender<T> {
    fn try_send(&mut self, message: T) -> TrySendResult<T>;
}

pub type BoxedSender<T> = Box<dyn Sender<T>>;

impl<T: 'static> Sender<T> for Box<dyn Sender<T>> {
    fn try_send(&mut self, message: T) -> TrySendResult<T> {
        (**self).try_send(message)
    }
}

pub mod error {
    use thiserror::Error as ThisError;

    #[derive(Debug, ThisError)]
    #[error("failure when trying to send via channel")]
    pub enum TrySendError<T> {
        #[error("channel is full")]
        Full(T),
        #[error("channel got disconnected")]
        Disconnected(T),
    }

    #[derive(Debug, ThisError)]
    #[error("failure when trying to receive via channel")]
    pub enum TryRecvError {
        #[error("channel is empty")]
        Empty,
        #[error("channel got disconnected")]
        Disconnected,
        #[error("receiver lagged behind {0} messages")]
        Lagged(u64),
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

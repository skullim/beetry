mod channel;
mod leaf;
mod node;
mod root;
mod task;
mod tree;

pub use leaf::{
    Action, ActionBehavior, BoxActionBehavior, BoxConditionBehavior, Condition, ConditionBehavior,
};
#[cfg(any(test, feature = "mock"))]
pub use node::MockNode;
pub use node::{BoxNode, Node, NonEmptyNodes};

pub use root::Root;
pub use tree::{PeriodicTick, Ticker, Tree, TreeEngine, TreeEngineError};
pub type PeriodicTicker = Ticker<PeriodicTick>;

pub use task::{
    AbortTask, BoxTaskFuture, ExecutorConcept, NodeTask, QueryTask, RegisterTask, Task,
    TaskDescription, TaskHandle, TaskStatus,
};

pub use channel::{
    BoxReceiver, BoxSender, MessageHash, Receiver, Sender, TryRecvResult, TrySendResult, error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TickStatus {
    Failure,
    Success,
    Running,
}

impl TickStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Success | Self::Failure)
    }
}

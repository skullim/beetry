//! # Beetry Core
//!
//! This crate is an internal Beetry implementation crate and is not considered
//! part of the public API. For public APIs, use the `beetry` crate.
//!
//! `beetry-core` defines foundational behavior tree framework traits and
//! concepts.
//!
//! - [`Node`]: the trait every executable tree node implements
//! - [`Ticker`]: a driver that advances a tree from an external tick source
//! - [`ConditionBehavior`]: the behavior contract used by [`Condition`] leaf
//!   nodes
//! - [`ActionBehavior`]: the behavior contract used by [`Action`] leaf nodes
//! - [`Sender`] and [`Receiver`]: the channel-facing traits used for message
//!   passing between nodes
//!
//! ## `Node`
//!
//! [`Node`] is the central runtime trait in Beetry. Every control node,
//! decorator, condition, action, or whole tree is ultimately driven through the
//! same interface. This keeps the tree runtime uniform: composite nodes can
//! schedule children without needing a different protocol from leaf nodes, and
//! higher-level drivers only need to understand the node lifecycle.
//!
//! ## `Ticker`
//!
//! [`Ticker`] drives a tree from an external tick source.
//!
//! Because [`Ticker`] can be built from a custom stream, each application can
//! define its own ticking mechanism without changing the tree implementation.
//!
//! ## Channel
//!
//! Beetry models communication between nodes through channel abstractions
//! rather than through a shared blackboard in the core runtime.
//!
//! [`Sender`] and [`Receiver`] define the minimal non-blocking contracts for
//! sending and receiving typed messages.
//!
//! ## Leaf behavior concepts
//!
//! ### `Condition`
//!
//! [`ConditionBehavior`] is the minimal synchronous leaf contract. It is used
//! by [`Condition`], which wraps the behavior and maps its condition evaluation
//! to [`TickStatus::Success`] or [`TickStatus::Failure`].
//!
//! ### `Action`
//!
//! [`ActionBehavior`] is the asynchronous leaf contract used by [`Action`].
//! [`Action`] wraps the behavior and uses it to construct a [`NodeTask`] that
//! can be registered with an executor instead of directly returning a
//! [`TickStatus`] from `tick`.
//!
//! Note: Action abort is currently implemented as a blocking operation: it
//! signals the executor to abort the task, then waits until the task reaches a
//! terminal state.

mod channel;
pub mod leaf;
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
pub use tree::{PeriodicTick, Ticker, TickerError, Tree};
pub type PeriodicTicker = Ticker<PeriodicTick>;

pub use channel::{BoxReceiver, BoxSender, Receiver, Sender, TryRecvResult, TrySendResult, error};
pub use task::{
    AbortTask, BoxTaskFuture, ExecutorConcept, NodeTask, QueryTask, RegisterTask, Task,
    TaskDescription, TaskHandle, TaskStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TickStatus {
    Failure,
    Success,
    Running,
}

impl TickStatus {
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Success | Self::Failure)
    }
}

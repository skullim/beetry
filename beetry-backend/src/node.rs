mod control;
mod decorator;
mod leaf;
#[cfg(test)]
mod mock;
mod traced;

mod nonempty;
pub use nonempty::NonEmptyNodes;

#[cfg(test)]
pub(crate) use mock::test as mock_test;

pub use control::{Fallback, Parallel, Sequence};
pub(crate) use leaf::{Action, Condition};
pub use leaf::{ActionBehavior, ConditionBehavior};
pub(crate) use traced::TracedNode;

use crate::status::TreeStatus;
use strum_macros::Display;

#[cfg(test)]
use mockall::automock;
#[cfg_attr(test, automock)]
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

pub trait Identifiable {
    fn id(&self) -> &NodeIdentifier;
}

pub trait IdentifiableNode: Node + Identifiable {}
impl<N> IdentifiableNode for N where N: Node + Identifiable {}

#[derive(Debug, Clone)]
pub(crate) struct NodeIdentifier {
    ty: NodeType,
}

impl NodeIdentifier {
    pub(crate) fn new(ty: NodeType) -> Self {
        Self { ty }
    }
}

impl std::fmt::Display for NodeIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{type: {}}}", self.ty)
    }
}

#[derive(Debug, Display, Clone, Copy)]
pub(crate) enum NodeType {
    Root,
    Sequence,
    Fallback,
    Parallel,
    Action,
    Condition,
}

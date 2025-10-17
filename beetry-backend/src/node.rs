mod control;
mod decorator;
mod leaf;
mod root;

pub use control::{Fallback, Parallel, Sequence};
pub use leaf::{Action, ActionBehavior, Condition, ConditionBehavior};
pub use root::Root;

mod nonempty;
pub use nonempty::NonEmptyNodes;

#[cfg(test)]
mod mock;
#[cfg(test)]
pub(crate) use mock::test as mock_test;

use crate::status::TreeStatus;

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

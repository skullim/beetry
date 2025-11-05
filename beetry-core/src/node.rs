use crate::TickStatus;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait Node {
    fn tick(&mut self) -> TickStatus;
    /// reset given node to its default state:
    /// - should only be called by the behavior tree (root)
    /// - should not block the thread and ideally be finished during single tick
    fn reset(&mut self) {}

    /// interface to abort running tasks. The work is to be done by the leaf (action) nodes only, as control or decorator nodes are not scheduled on the executor
    fn abort(&mut self) {}
}

pub type BoxNode = Box<dyn Node>;

pub trait ControlNode: Node {}

impl Node for BoxNode {
    fn tick(&mut self) -> TickStatus {
        (**self).tick()
    }

    fn reset(&mut self) {
        (**self).reset()
    }

    fn abort(&mut self) {
        (**self).abort()
    }
}

mod ticker;

pub use ticker::{Error as TickerError, PeriodicTick, Ticker};

use crate::{Node, TickStatus, root::Root};

/// Behavior tree rooted at a [`Root`] node.
pub struct Tree<N> {
    root: Root<N>,
}

impl<N> Tree<N>
where
    N: Node,
{
    /// Create a new tree from the given root node.
    pub fn new(root: Root<N>) -> Self {
        Self { root }
    }
}

impl<N> Node for Tree<N>
where
    N: Node,
{
    fn reset(&mut self) {
        self.root.reset();
    }

    fn tick(&mut self) -> TickStatus {
        self.root.tick()
    }
}

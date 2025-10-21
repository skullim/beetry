mod engine;
mod ticker;

pub use engine::TreeEngine;
pub use ticker::Ticker;

use crate::{Node, TickStatus, root::Root};

pub struct BehaviorTree<N> {
    root: Root<N>,
}

impl<N> BehaviorTree<N>
where
    N: Node,
{
    pub fn new(root: Root<N>) -> Self {
        Self { root }
    }
}

impl<N> Node for BehaviorTree<N>
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

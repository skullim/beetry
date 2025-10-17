mod builder;
mod engine;
mod ticker;

pub use builder::Builder;
pub use engine::TreeEngine;
pub use ticker::Ticker;

use crate::{
    node::{Node, Root},
    status::TreeStatus,
};

pub struct BehaviorTree<N>
where
    N: Node,
{
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

    fn tick(&mut self) -> TreeStatus {
        self.root.tick()
    }
}

mod ticker;

pub use ticker::{Error as TickerError, PeriodicTick, Ticker};

use crate::root::Root;
use crate::{Node, TickStatus};

pub struct Tree<N> {
    root: Root<N>,
}

impl<N> Tree<N>
where
    N: Node,
{
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

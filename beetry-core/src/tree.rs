mod engine;
mod ticker;

pub use self::Error as TreeEngineError;
pub use engine::TreeEngine;
pub use ticker::{PeriodicTick, Ticker};

use crate::root::Root;
use crate::{Node, TickStatus};
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("tick source was exhausted before tree reached terminal state")]
    TickSourceExhausted,
    #[error("executor failed before tree reached terminal state: {0}")]
    ExecutorFailure(String),
}

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

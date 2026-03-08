use crate::tree::Error;
use crate::{Node, PeriodicTicker, TickStatus, Tree};

pub struct TreeEngine<N> {
    tree: Tree<N>,
}

impl<N> TreeEngine<N>
where
    N: Node,
{
    pub fn new(tree: Tree<N>) -> Self {
        Self { tree }
    }

    pub async fn tick_till_terminal(
        &mut self,
        mut ticker: PeriodicTicker,
    ) -> Result<TickStatus, Error> {
        ticker.tick_till_terminal(&mut self.tree).await
    }
}

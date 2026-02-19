use crate::task::ExecutorConcept;
use crate::tree::Error;
use crate::{Node, PeriodicTicker, TickStatus, Tree};

pub struct TreeEngine<N> {
    tree: Tree<N>,
}

impl<N> TreeEngine<N>
where
    N: Node,
{
    pub fn new(tree: Tree<N>) -> TreeEngine<N> {
        TreeEngine { tree }
    }

    pub async fn tick_till_terminal<E>(
        &mut self,
        mut ticker: PeriodicTicker,
        executor: &mut E,
    ) -> Result<TickStatus, Error>
    where
        E: ExecutorConcept,
    {
        tokio::select! {
            status = ticker.tick_till_terminal(&mut self.tree)
            => {status}
            result = executor.run() => {
                match result {
                    Ok(()) => Err(Error::ExecutorFailure(
                        "executor terminated before tree reached terminal state".to_string(),
                    )),
                    Err(err) => Err(Error::ExecutorFailure(err.to_string())),
                }
            }
        }
    }
}

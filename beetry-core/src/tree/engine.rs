use crate::{BehaviorTreeTicker, Node, TickStatus, Tree, task::ExecutorConcept};
use state_shift::{impl_state, type_state};

#[type_state(states = (Initial, TickerSet, TreeSet), slots = (Initial))]
pub struct TreeEngine<N> {
    ticker: Option<BehaviorTreeTicker>,
    tree: Option<Tree<N>>,
}

#[impl_state]
impl<N> TreeEngine<N>
where
    N: Node,
{
    #[require(Initial)]
    pub fn new() -> TreeEngine<N> {
        TreeEngine {
            ticker: None,
            tree: None,
        }
    }

    #[require(Initial)]
    #[switch_to(TickerSet)]
    pub fn set_ticker(self, ticker: BehaviorTreeTicker) -> TreeEngine<N> {
        TreeEngine {
            ticker: Some(ticker),
            tree: None,
        }
    }

    #[require(TickerSet)]
    #[switch_to(TreeSet)]
    pub fn set_tree(self, tree: Tree<N>) -> TreeEngine<N> {
        TreeEngine {
            ticker: self.ticker,
            tree: Some(tree),
        }
    }

    #[require(TreeSet)]
    pub async fn tick_till_terminal<E>(&mut self, executor: &mut E) -> TickStatus
    where
        E: ExecutorConcept,
    {
        tokio::select! {
            status = self.ticker.as_mut().expect("type system guarantees ticker is set").tick_till_terminal(self.tree.as_mut().expect("type system guarantee tree is set"))
            => {status}
            result = executor.run() => {
                result.unwrap();
                TickStatus::Failure
            }
        }
    }
}

use std::sync::Arc;

use super::Builder;
use crate::{
    BehaviorTree, BehaviorTreeBuilder, BehaviorTreeTicker, Node, TreeStatus,
    task::{Executor, Registry},
};

use state_shift::{impl_state, type_state};
use tokio::sync::mpsc::channel;

#[type_state(states = (Initial, TickerSet, TreeSet), slots = (Initial))]
pub struct TreeEngine<N>
where
    N: Node,
{
    executor: Executor,
    builder: Builder,
    ticker: Option<BehaviorTreeTicker>,
    tree: Option<BehaviorTree<N>>,
}

#[impl_state]
impl<N> TreeEngine<N>
where
    N: Node,
{
    #[require(Initial)]
    pub fn new() -> TreeEngine<N> {
        let (sender, recv) = channel(8);
        let executor = Executor::new(recv);
        let builder = BehaviorTreeBuilder::new(Arc::new(Registry::new(sender)));

        TreeEngine {
            executor,
            builder,
            ticker: None,
            tree: None,
        }
    }

    #[require(Initial)]
    pub fn tree_builder(&self) -> &Builder {
        &self.builder
    }

    #[require(Initial)]
    #[switch_to(TickerSet)]
    pub fn set_ticker(self, ticker: BehaviorTreeTicker) -> TreeEngine<N> {
        TreeEngine {
            executor: self.executor,
            builder: self.builder,
            ticker: Some(ticker),
            tree: None,
        }
    }

    #[require(TickerSet)]
    #[switch_to(TreeSet)]
    pub fn set_tree(self, tree: BehaviorTree<N>) -> TreeEngine<N> {
        TreeEngine {
            executor: self.executor,
            builder: self.builder,
            ticker: self.ticker,
            tree: Some(tree),
        }
    }

    #[require(TreeSet)]
    pub async fn tick_till_terminal(&mut self) -> TreeStatus {
        tokio::select! {
            status = self.ticker.as_mut().expect("type system guarantees ticker is set").tick_till_terminal(self.tree.as_mut().expect("type system guarantee tree is set"))
            => {status}
            result = self.executor.drive() => {
                result.unwrap();
                TreeStatus::Failure
            }
        }
    }
}

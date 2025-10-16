use std::sync::Arc;

use super::Builder;
use crate::{
    BehaviorTreeBuilder, BehaviorTreeTicker, TreeStatus,
    task::{Executor, Registry},
};

use state_shift::{impl_state, type_state};
use tokio::sync::mpsc::channel;

#[type_state(states = (Initial, TickerSet), slots = (Initial))]
pub struct TreeEngine {
    executor: Executor,
    builder: Builder,
    ticker: Option<BehaviorTreeTicker>,
}

#[impl_state]
impl TreeEngine {
    #[require(Initial)]
    pub fn new() -> TreeEngine {
        let (sender, recv) = channel(8);
        let executor = Executor::new(recv);
        let builder = BehaviorTreeBuilder::new(Arc::new(Registry::new(sender)));

        TreeEngine {
            executor,
            builder,
            ticker: None,
        }
    }

    #[require(Initial)]
    pub fn tree_builder(&self) -> &Builder {
        &self.builder
    }

    #[require(Initial)]
    #[switch_to(TickerSet)]
    pub fn set_ticker(self, ticker: BehaviorTreeTicker) -> TreeEngine {
        TreeEngine {
            executor: self.executor,
            builder: self.builder,
            ticker: Some(ticker),
        }
    }

    #[require(TickerSet)]
    pub async fn tick_till_terminal(&mut self) -> TreeStatus {
        tokio::select! {
            status = self.ticker.as_mut().expect("type system guarantees set").tick_till_terminal()
            => {status}
            result = self.executor.drive() => {
                result.unwrap();
                TreeStatus::Failure
            }
        }
    }
}

use std::time::Duration;

use tokio::time::{Instant, Interval, MissedTickBehavior};
use tracing::{debug, instrument, warn};

use crate::Node;
use crate::{BehaviorTree, TickStatus};

pub struct Ticker {
    interval: Interval,
}

struct TickHealthMonitor {
    last: Option<Instant>,
    expected: Duration,
}

impl TickHealthMonitor {
    fn new(expected: Duration) -> Self {
        Self {
            last: None,
            expected,
        }
    }

    fn monitor(&mut self, current: Instant) {
        if let Some(last) = self.last {
            let took = current.duration_since(last);
            if took > self.expected {
                warn!(
                    "exceeded tick interval, it took {took:?}, expected {:?}",
                    self.expected
                );
            }
        }
        self.last = Some(current);
    }
}

impl Ticker {
    pub fn new(period: Duration) -> Self {
        let mut interval = tokio::time::interval(period);
        // @todo let client set the tick behavior
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        Self { interval }
    }

    #[instrument(skip_all)]
    pub async fn tick_till_terminal<N>(&mut self, tree: &mut BehaviorTree<N>) -> TickStatus
    where
        N: Node,
    {
        // start warning if tick takes twice as much time as it should
        let mut tick_monitor = TickHealthMonitor::new(2 * self.interval.period());
        loop {
            // @todo might actually allow user to specify multiple conditions when the tick can happen
            // e.g. one might want to tick the tree 'faster' as some extraordinary situation occurred.
            // In this case the next tick should be executed without waiting for current tick to timeout.
            let current = self.interval.tick().await;
            tick_monitor.monitor(current);
            let status = tree.tick();
            debug!("ticked bt yielded status: {status:?}");
            match status {
                status @ (TickStatus::Failure | TickStatus::Success) => {
                    debug!("finished executing bt with status: {status:?}");
                    return status;
                }
                TickStatus::Running => continue,
            }
        }
    }
}

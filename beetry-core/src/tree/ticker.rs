use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{Stream, future::poll_fn};
use thiserror::Error as ThisError;
use tokio::time::MissedTickBehavior;

use crate::{Node, TickStatus};

/// Ticks a behavior tree using an external tick source.
///
/// `Ticker` is built from any `Stream<Item = TickSignal>`, so callers can
/// define their own ticking mechanism. A tick source can be periodic with
/// [`PeriodicTick`], event-driven from an external stream, or a hybrid of
/// time-based and signal-based wakeups.
pub struct Ticker<S> {
    stream: Pin<Box<S>>,
}

pub type TickSignal = ();

/// Errors that can occur while ticking a tree from a tick source.
#[derive(Debug, ThisError)]
pub enum Error {
    /// The tick source ended before the tree reached a terminal status.
    #[error("tick source was exhausted")]
    SourceExhausted,
}

impl<S> Ticker<S>
where
    S: Stream<Item = TickSignal>,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream: Box::pin(stream),
        }
    }

    pub async fn tick_till_terminal(&mut self, tree: &mut impl Node) -> Result<TickStatus, Error> {
        while poll_fn(|cx| self.stream.as_mut().poll_next(cx))
            .await
            .is_some()
        {
            match tree.tick() {
                TickStatus::Running => {}
                s @ (TickStatus::Success | TickStatus::Failure) => return Ok(s),
            }
        }
        Err(Error::SourceExhausted)
    }
}

/// Built-in periodic tick source. Useful as the default in most applications.
pub struct PeriodicTick {
    interval: tokio::time::Interval,
}

impl PeriodicTick {
    #[must_use]
    pub fn new(period: Duration) -> Self {
        Self {
            interval: tokio::time::interval(period),
        }
    }

    pub fn with_missed_tick_behavior(&mut self, behavior: MissedTickBehavior) {
        self.interval.set_missed_tick_behavior(behavior);
    }
}

impl Stream for PeriodicTick {
    type Item = TickSignal;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().interval.poll_tick(cx).map(|_| Some(()))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::stream;

    use super::*;
    use crate::{MockNode, TickStatus};

    #[tokio::test]
    async fn periodic_tick_yields_terminal_status() {
        let mut node = MockNode::new();
        node.expect_tick().once().return_const(TickStatus::Success);

        let mut ticker = Ticker::new(PeriodicTick::new(Duration::from_millis(1)));
        let status = ticker
            .tick_till_terminal(&mut node)
            .await
            .expect("periodic stream should tick at least once");

        assert_eq!(status, TickStatus::Success);
    }

    #[tokio::test]
    async fn terminal_status_precedes_source_exhaustion() {
        let mut node = MockNode::new();
        let mut statuses = [TickStatus::Running, TickStatus::Success].into_iter();
        node.expect_tick()
            .times(2)
            .returning(move || statuses.next().expect("status sequence configured"));

        let mut ticker = Ticker::new(stream::iter([(), (), ()]));
        let status = ticker
            .tick_till_terminal(&mut node)
            .await
            .expect("tree reaches terminal status before source exhaustion");

        assert_eq!(status, TickStatus::Success);
    }

    #[tokio::test]
    async fn tick_source_exhausted() {
        let mut node = MockNode::new();
        node.expect_tick()
            .times(2)
            .return_const(TickStatus::Running);

        let mut ticker = Ticker::new(stream::iter([(), ()]));
        let result = ticker.tick_till_terminal(&mut node).await;
        assert!(matches!(result, Err(Error::SourceExhausted)));
    }
}

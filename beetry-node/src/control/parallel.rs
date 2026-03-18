use beetry_core::{Node, NonEmptyNodes, TickStatus};
use bon::Builder;

use crate::{Indices, control::RunningNodesAborter};

/// Threshold configuration for [`Parallel`].
#[derive(Debug, Clone, Copy, Builder)]
#[cfg_attr(feature = "plugin", derive(serde::Deserialize))]
pub struct ParallelParams {
    /// Number of children that must succeed for the node to succeed.
    pub success_count: u16,
    /// Number of children that must fail for the node to fail.
    pub failure_count: u16,
}

/// Ticks all children and resolves once the configured success or failure
/// threshold is reached.
pub struct Parallel {
    nodes: NonEmptyNodes,
    aborter: RunningNodesAborter,
    params: ParallelParams,
}

impl Parallel {
    #[must_use]
    pub fn new(nodes: impl Into<NonEmptyNodes>, params: ParallelParams) -> Self {
        Self {
            nodes: nodes.into(),
            aborter: RunningNodesAborter::new(),
            params,
        }
    }
}

impl Node for Parallel {
    fn tick(&mut self) -> TickStatus {
        let aborter: &mut RunningNodesAborter = &mut self.aborter;
        let mut success_count = 0_u16;
        let mut failure_count = 0_u16;

        for idx in self.nodes.indices() {
            let node = &mut self.nodes[idx];
            match node.tick() {
                TickStatus::Success => {
                    success_count += 1;
                    aborter.untrack(idx);
                }
                TickStatus::Running => {
                    aborter.track(idx);
                }
                TickStatus::Failure => {
                    failure_count += 1;
                    aborter.untrack(idx);
                }
            }
        }

        if failure_count >= self.params.failure_count {
            aborter.abort_all(&mut self.nodes);
            return TickStatus::Failure;
        }

        if success_count >= self.params.success_count {
            aborter.abort_all(&mut self.nodes);
            return TickStatus::Success;
        }

        if aborter.is_any_tracked() {
            TickStatus::Running
        } else {
            TickStatus::Failure
        }
    }

    fn abort(&mut self) {
        self.aborter.clear();
        for node in &mut self.nodes {
            node.abort();
        }
    }

    fn reset(&mut self) {
        self.aborter.clear();
        for node in &mut self.nodes {
            node.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_test::{boxed, mock_returns};

    fn params(success_count: u16, failure_count: u16) -> ParallelParams {
        ParallelParams::builder()
            .success_count(success_count)
            .failure_count(failure_count)
            .build()
    }

    #[test]
    fn succeeds_on_success_threshold() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Success])),
            boxed(mock_returns([TickStatus::Success])),
            boxed(mock_returns([TickStatus::Failure])),
        ]);
        let mut pl = Parallel::new(nodes, params(2, 3));

        assert_eq!(pl.tick(), TickStatus::Success);
    }

    #[test]
    fn fails_on_failure_threshold() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Failure])),
            boxed(mock_returns([TickStatus::Failure])),
            boxed(mock_returns([TickStatus::Success])),
        ]);
        let mut pl = Parallel::new(nodes, params(3, 2));

        assert_eq!(pl.tick(), TickStatus::Failure);
    }

    #[test]
    fn returns_running_when_no_threshold_reached_and_any_running() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Success])),
            boxed(mock_returns([TickStatus::Running])),
            boxed(mock_returns([TickStatus::Failure])),
        ]);
        let mut pl = Parallel::new(nodes, params(2, 2));

        assert_eq!(pl.tick(), TickStatus::Running);
    }

    #[test]
    fn fails_when_all_terminal_and_no_threshold_reached() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Success])),
            boxed(mock_returns([TickStatus::Failure])),
            boxed(mock_returns([TickStatus::Failure])),
        ]);
        let mut pl = Parallel::new(nodes, params(3, 3));

        assert_eq!(pl.tick(), TickStatus::Failure);
    }

    #[test]
    fn aborts_tracked_nodes_when_threshold_reached() {
        let mut m1 = mock_returns([TickStatus::Running]);
        let m2 = mock_returns([TickStatus::Success]);
        let m3 = mock_returns([TickStatus::Success]);

        m1.expect_abort().once().return_const(());

        let nodes = NonEmptyNodes::from([boxed(m1), boxed(m2), boxed(m3)]);
        let mut pl = Parallel::new(nodes, params(2, 3));

        assert_eq!(pl.tick(), TickStatus::Success);
    }
}

use beetry_core::{Node, NonEmptyNodes, TickStatus};

use crate::{Indices, control::RunningNodesAborter};

/// Ticks children from left to right until one succeeds or is still running.
pub struct Fallback {
    nodes: NonEmptyNodes,
    aborter: RunningNodesAborter,
}

impl Fallback {
    #[must_use]
    pub fn new(nodes: NonEmptyNodes) -> Self {
        Self {
            nodes,
            aborter: RunningNodesAborter::new(),
        }
    }
}

impl Node for Fallback {
    fn tick(&mut self) -> TickStatus {
        let aborter = &mut self.aborter;
        for idx in self.nodes.indices() {
            let node = &mut self.nodes[idx];
            match node.tick() {
                TickStatus::Failure => {
                    aborter.untrack(idx);
                }
                TickStatus::Running => {
                    aborter.abort_if_other_running(&mut self.nodes, idx);
                    aborter.track(idx);
                    return TickStatus::Running;
                }
                TickStatus::Success => {
                    aborter.abort_if_other_running(&mut self.nodes, idx);
                    return TickStatus::Success;
                }
            }
        }
        TickStatus::Failure
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
    use beetry_core::{Node, TickStatus};

    use super::*;
    use crate::mock_test::{boxed, mock_returns};

    #[test]
    fn success_with_first_success() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Failure])),
            boxed(mock_returns([TickStatus::Success])),
            boxed(mock_returns([])),
        ]);
        let mut fb = Fallback::new(nodes);

        assert_eq!(fb.tick(), TickStatus::Success);
    }

    #[test]
    fn running_with_first_running() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Failure])),
            boxed(mock_returns([TickStatus::Running])),
            boxed(mock_returns([])),
        ]);
        let mut fb = Fallback::new(nodes);
        assert_eq!(fb.tick(), TickStatus::Running);
    }

    #[test]
    fn failure_with_all_failed() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Failure])),
            boxed(mock_returns([TickStatus::Failure])),
        ]);
        let mut fb = Fallback::new(nodes);

        assert_eq!(fb.tick(), TickStatus::Failure);
    }

    #[test]
    fn resets_running() {
        let m1 = mock_returns([
            TickStatus::Failure,
            TickStatus::Failure,
            TickStatus::Running,
        ]);
        let mut m2 = mock_returns([TickStatus::Failure, TickStatus::Running]);
        let mut m3 = mock_returns([TickStatus::Running]);
        m2.expect_abort().once().return_const(());
        m3.expect_abort().once().return_const(());

        let nodes = NonEmptyNodes::from([boxed(m1), boxed(m2), boxed(m3)]);
        let mut fb = Fallback::new(nodes);

        assert_eq!(fb.tick(), TickStatus::Running);
        assert_eq!(fb.tick(), TickStatus::Running);
        assert_eq!(fb.tick(), TickStatus::Running);
    }
}

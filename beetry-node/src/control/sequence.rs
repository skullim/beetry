use beetry_core::{Node, NonEmptyNodes, TickStatus};

use crate::{Indices, control::RunningNodesAborter};

/// Control node that requires all children to succeed in order.
///
/// `Sequence` ticks children from left to right on each tick:
///
/// - returns [`TickStatus::Failure`] as soon as a child fails
/// - returns [`TickStatus::Running`] as soon as a child is still running
/// - returns [`TickStatus::Success`] only if every child succeeds on the same
///   tick
///
/// This variant does not remember which child was previously running, so the
/// next tick starts again from the first child.
pub struct Sequence {
    nodes: NonEmptyNodes,
    aborter: RunningNodesAborter,
}

impl Sequence {
    #[must_use]
    pub fn new(nodes: impl Into<NonEmptyNodes>) -> Self {
        Self {
            nodes: nodes.into(),
            aborter: RunningNodesAborter::new(),
        }
    }
}

impl Node for Sequence {
    fn tick(&mut self) -> TickStatus {
        let aborter = &mut self.aborter;
        for idx in self.nodes.indices() {
            let node = &mut self.nodes[idx];
            match node.tick() {
                TickStatus::Success => {
                    aborter.untrack(idx);
                }
                TickStatus::Running => {
                    aborter.abort_if_other_running(&mut self.nodes, idx);
                    aborter.track(idx);
                    return TickStatus::Running;
                }

                TickStatus::Failure => {
                    aborter.abort_if_other_running(&mut self.nodes, idx);
                    return TickStatus::Failure;
                }
            }
        }
        TickStatus::Success
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

/// Memory-based [`Sequence`] variant.
///
/// Unlike [`Sequence`], `MemorySequence` remembers the last running child and
/// resumes from it on the next tick instead of restarting from the beginning.
pub struct MemorySequence {
    nodes: NonEmptyNodes,
    running_idx: Option<usize>,
}

impl MemorySequence {
    pub fn new(nodes: impl Into<NonEmptyNodes>) -> Self {
        Self {
            nodes: nodes.into(),
            running_idx: None,
        }
    }
}

impl Node for MemorySequence {
    fn tick(&mut self) -> TickStatus {
        let start_idx = self.running_idx.take().unwrap_or(0);
        for idx in start_idx..self.nodes.len().into() {
            let node = &mut self.nodes[idx];
            match node.tick() {
                TickStatus::Success => {}
                TickStatus::Running => {
                    self.running_idx = Some(idx);
                    return TickStatus::Running;
                }
                TickStatus::Failure => {
                    return TickStatus::Failure;
                }
            }
        }
        TickStatus::Success
    }

    fn abort(&mut self) {
        self.running_idx = None;
        for node in &mut self.nodes {
            node.abort();
        }
    }

    fn reset(&mut self) {
        self.running_idx = None;
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
    fn success_with_all_success() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Success])),
            boxed(mock_returns([TickStatus::Success])),
        ]);
        let mut sq = Sequence::new(nodes);

        assert_eq!(sq.tick(), TickStatus::Success);
    }

    #[test]
    fn running_with_first_running() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Success])),
            boxed(mock_returns([TickStatus::Running])),
            boxed(mock_returns([])),
        ]);
        let mut sq = Sequence::new(nodes);
        assert_eq!(sq.tick(), TickStatus::Running);
    }

    #[test]
    fn failure_with_first_failed() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Success])),
            boxed(mock_returns([TickStatus::Failure])),
            boxed(mock_returns([])),
        ]);
        let mut sq = Sequence::new(nodes);

        assert_eq!(sq.tick(), TickStatus::Failure);
    }

    #[test]
    fn resets_running() {
        let m1 = mock_returns([
            TickStatus::Success,
            TickStatus::Success,
            TickStatus::Running,
        ]);
        let mut m2 = mock_returns([TickStatus::Success, TickStatus::Running]);
        let mut m3 = mock_returns([TickStatus::Running]);
        m2.expect_abort().once().return_const(());
        m3.expect_abort().once().return_const(());

        let nodes = NonEmptyNodes::from([boxed(m1), boxed(m2), boxed(m3)]);
        let mut sq = Sequence::new(nodes);

        assert_eq!(sq.tick(), TickStatus::Running);
        assert_eq!(sq.tick(), TickStatus::Running);
        assert_eq!(sq.tick(), TickStatus::Running);
    }

    #[test]
    fn mem_sequence_resumes_from_running_child() {
        let m1 = mock_returns([TickStatus::Success]);
        let m2 = mock_returns([
            TickStatus::Running,
            TickStatus::Running,
            TickStatus::Success,
        ]);
        let m3 = mock_returns([TickStatus::Success]);

        let nodes = NonEmptyNodes::from([boxed(m1), boxed(m2), boxed(m3)]);
        let mut msq = MemorySequence::new(nodes);

        assert_eq!(msq.tick(), TickStatus::Running);
        assert_eq!(msq.tick(), TickStatus::Running);
        assert_eq!(msq.tick(), TickStatus::Success);
    }

    #[test]
    fn mem_sequence_reset_clears_memory() {
        let mut m1 = mock_returns([TickStatus::Success, TickStatus::Success]);
        m1.expect_reset().once().return_const(());
        let mut m2 = mock_returns([TickStatus::Running, TickStatus::Running]);
        m2.expect_reset().once().return_const(());

        let nodes = NonEmptyNodes::from([boxed(m1), boxed(m2)]);
        let mut msq = MemorySequence::new(nodes);

        assert_eq!(msq.tick(), TickStatus::Running);
        msq.reset();
        assert_eq!(msq.tick(), TickStatus::Running);
    }
}

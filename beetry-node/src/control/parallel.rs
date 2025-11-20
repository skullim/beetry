use crate::Indices;
use crate::control::RunningNodesAborter;
use beetry_core::{ControlNode, Node, NonEmptyNodes, TickStatus};

/// Parallel node succeeds when all nodes succeed
pub struct Parallel {
    nodes: NonEmptyNodes,
    aborter: RunningNodesAborter,
}

impl Parallel {
    pub fn new(nodes: impl Into<NonEmptyNodes>) -> Self {
        Self {
            nodes: nodes.into(),
            aborter: RunningNodesAborter::new(),
        }
    }
}

impl Node for Parallel {
    fn tick(&mut self) -> TickStatus {
        let aborter: &mut RunningNodesAborter = &mut self.aborter;
        for idx in self.nodes.indices() {
            let node = &mut self.nodes[idx];
            match node.tick() {
                TickStatus::Success => {
                    aborter.untrack(idx);
                }
                TickStatus::Running => {
                    aborter.track(idx);
                }
                TickStatus::Failure => {
                    aborter.untrack(idx);
                    aborter.abort_all(&mut self.nodes);
                    return TickStatus::Failure;
                }
            }
        }

        if aborter.is_any_tracked() {
            TickStatus::Running
        } else {
            TickStatus::Success
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

impl ControlNode for Parallel {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_test::{boxed, mock, tick_returns};
    use beetry_core::{MockNode, Node, TickStatus};

    #[test]
    fn success_with_all_success() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TickStatus::Success).times(1).call()),
            boxed(mock().status(TickStatus::Success).times(1).call()),
        ]);
        let mut pl = Parallel::new(nodes);

        assert_eq!(pl.tick(), TickStatus::Success);
    }

    #[test]
    fn running_with_any_running() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TickStatus::Success).times(1).call()),
            boxed(mock().status(TickStatus::Running).times(1).call()),
            boxed(mock().status(TickStatus::Running).times(1).call()),
        ]);
        let mut pl = Parallel::new(nodes);
        assert_eq!(pl.tick(), TickStatus::Running);
    }

    #[test]
    fn failure_with_any_failed() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TickStatus::Success).times(1).call()),
            boxed(mock().status(TickStatus::Failure).times(1).call()),
            boxed(mock().status(TickStatus::Success).times(0).call()),
        ]);
        let mut pl = Parallel::new(nodes);

        assert_eq!(pl.tick(), TickStatus::Failure);
    }

    #[test]
    fn resets_running() {
        let (mut m1, mut m2, mut m3) = (MockNode::new(), MockNode::new(), MockNode::new());
        tick_returns(
            &mut m1,
            vec![
                TickStatus::Running,
                TickStatus::Running,
                TickStatus::Failure,
            ],
        );
        tick_returns(&mut m2, vec![TickStatus::Running, TickStatus::Running]);
        tick_returns(&mut m3, vec![TickStatus::Running, TickStatus::Running]);

        m2.expect_abort().once().return_const(());
        m3.expect_abort().once().return_const(());

        let nodes = NonEmptyNodes::from([boxed(m1), boxed(m2), boxed(m3)]);
        let mut pl = Parallel::new(nodes);

        assert_eq!(pl.tick(), TickStatus::Running);
        assert_eq!(pl.tick(), TickStatus::Running);
        assert_eq!(pl.tick(), TickStatus::Failure);
    }
}

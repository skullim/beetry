use crate::Indices;
use crate::control::RunningNodesAborter;
use beetry_core::{ControlNode, Node, NonEmptyNodes, TickStatus};

pub struct Sequence {
    nodes: NonEmptyNodes,
    aborter: RunningNodesAborter,
}

impl Sequence {
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

impl ControlNode for Sequence {}

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
        let mut sq = Sequence::new(nodes);

        assert_eq!(sq.tick(), TickStatus::Success);
    }

    #[test]
    fn running_with_first_running() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TickStatus::Success).times(1).call()),
            boxed(mock().status(TickStatus::Running).times(1).call()),
            boxed(mock().status(TickStatus::Success).times(0).call()),
        ]);
        let mut sq = Sequence::new(nodes);
        assert_eq!(sq.tick(), TickStatus::Running);
    }

    #[test]
    fn failure_with_first_failed() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TickStatus::Success).times(1).call()),
            boxed(mock().status(TickStatus::Failure).times(1).call()),
            boxed(mock().status(TickStatus::Success).times(0).call()),
        ]);
        let mut sq = Sequence::new(nodes);

        assert_eq!(sq.tick(), TickStatus::Failure);
    }

    #[test]
    fn resets_running() {
        let (mut m1, mut m2, mut m3) = (MockNode::new(), MockNode::new(), MockNode::new());
        tick_returns(
            &mut m1,
            vec![
                TickStatus::Success,
                TickStatus::Success,
                TickStatus::Running,
            ],
        );
        tick_returns(&mut m2, vec![TickStatus::Success, TickStatus::Running]);
        tick_returns(&mut m3, vec![TickStatus::Running]);
        m2.expect_abort().once().return_const(());
        m3.expect_abort().once().return_const(());

        let nodes = NonEmptyNodes::from([boxed(m1), boxed(m2), boxed(m3)]);
        let mut sq = Sequence::new(nodes);

        assert_eq!(sq.tick(), TickStatus::Running);
        assert_eq!(sq.tick(), TickStatus::Running);
        assert_eq!(sq.tick(), TickStatus::Running);
    }
}

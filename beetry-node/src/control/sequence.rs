use beetry_core::{Node, TreeStatus};

use crate::{
    control::RunningNodesAborter,
    nonempty::{Indices, NonEmptyNodes},
};

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
    fn tick(&mut self) -> TreeStatus {
        let aborter = &mut self.aborter;
        for idx in self.nodes.indices() {
            let node = &mut self.nodes[idx];
            match node.tick() {
                TreeStatus::Success => {
                    aborter.untrack(idx);
                    continue;
                }
                TreeStatus::Running => {
                    aborter.abort_if_other_running(&mut self.nodes, idx);
                    aborter.track(idx);
                    return TreeStatus::Running;
                }

                TreeStatus::Failure => {
                    aborter.abort_if_other_running(&mut self.nodes, idx);
                    return TreeStatus::Failure;
                }
            }
        }
        TreeStatus::Success
    }

    fn abort(&mut self) {
        self.aborter.clear();
        for node in self.nodes.iter_mut() {
            node.abort();
        }
    }

    fn reset(&mut self) {
        self.aborter.clear();
        for node in self.nodes.iter_mut() {
            node.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_test::{boxed, mock, tick_returns};
    use beetry_core::{MockNode, Node, TreeStatus};

    #[test]
    fn success_with_all_success() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TreeStatus::Success).times(1).call()),
            boxed(mock().status(TreeStatus::Success).times(1).call()),
        ]);
        let mut sq = Sequence::new(nodes);

        assert_eq!(sq.tick(), TreeStatus::Success);
    }

    #[test]
    fn running_with_first_running() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TreeStatus::Success).times(1).call()),
            boxed(mock().status(TreeStatus::Running).times(1).call()),
            boxed(mock().status(TreeStatus::Success).times(0).call()),
        ]);
        let mut sq = Sequence::new(nodes);
        assert_eq!(sq.tick(), TreeStatus::Running);
    }

    #[test]
    fn failure_with_first_failed() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TreeStatus::Success).times(1).call()),
            boxed(mock().status(TreeStatus::Failure).times(1).call()),
            boxed(mock().status(TreeStatus::Success).times(0).call()),
        ]);
        let mut sq = Sequence::new(nodes);

        assert_eq!(sq.tick(), TreeStatus::Failure);
    }

    #[test]
    fn resets_running() {
        let (mut m1, mut m2, mut m3) = (MockNode::new(), MockNode::new(), MockNode::new());
        tick_returns(
            &mut m1,
            vec![
                TreeStatus::Success,
                TreeStatus::Success,
                TreeStatus::Running,
            ],
        );
        tick_returns(&mut m2, vec![TreeStatus::Success, TreeStatus::Running]);
        tick_returns(&mut m3, vec![TreeStatus::Running]);
        m2.expect_abort().once().return_const(());
        m3.expect_abort().once().return_const(());

        let nodes = NonEmptyNodes::from([boxed(m1), boxed(m2), boxed(m3)]);
        let mut sq = Sequence::new(nodes);

        assert_eq!(sq.tick(), TreeStatus::Running);
        assert_eq!(sq.tick(), TreeStatus::Running);
        assert_eq!(sq.tick(), TreeStatus::Running);
    }
}

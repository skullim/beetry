use nonempty::NonEmpty;

use crate::{
    node::{BoxedNode, Node, NodeIdentifier, NodeType, TracedNode, control::RunningNodesAborter},
    node_impl,
    status::TreeStatus,
};

struct SequenceControl {
    nodes: NonEmpty<BoxedNode>,
    aborter: RunningNodesAborter,
}

impl SequenceControl {
    fn new(nodes: NonEmpty<BoxedNode>) -> Self {
        Self {
            nodes,
            aborter: RunningNodesAborter::new(),
        }
    }
}

impl Node for SequenceControl {
    fn tick(&mut self) -> TreeStatus {
        let aborter = &mut self.aborter;
        for idx in 0..self.nodes.len() {
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

pub struct Sequence(TracedNode<SequenceControl>);
impl Sequence {
    pub fn new(nodes: NonEmpty<BoxedNode>) -> Self {
        Self(TracedNode::new(
            SequenceControl::new(nodes),
            NodeIdentifier::new(NodeType::Sequence),
        ))
    }
}
node_impl!(Sequence);

#[cfg(test)]
mod tests {
    use nonempty::nonempty;

    use super::*;
    use crate::node::MockNode;
    use crate::node::Node;
    use crate::node::mock_test::{boxed, mock, tick_returns};
    use crate::status::TreeStatus;

    #[test]
    fn success_with_all_success() {
        let nodes = nonempty![
            boxed(mock().status(TreeStatus::Success).times(1).call()),
            boxed(mock().status(TreeStatus::Success).times(1).call())
        ];
        let mut sq = Sequence::new(nodes);

        assert_eq!(sq.tick(), TreeStatus::Success);
    }

    #[test]
    fn running_with_first_running() {
        let nodes = nonempty![
            boxed(mock().status(TreeStatus::Success).times(1).call()),
            boxed(mock().status(TreeStatus::Running).times(1).call()),
            boxed(mock().status(TreeStatus::Success).times(0).call())
        ];
        let mut sq = Sequence::new(nodes);
        assert_eq!(sq.tick(), TreeStatus::Running);
    }

    #[test]
    fn failure_with_first_failed() {
        let nodes = nonempty![
            boxed(mock().status(TreeStatus::Success).times(1).call()),
            boxed(mock().status(TreeStatus::Failure).times(1).call()),
            boxed(mock().status(TreeStatus::Success).times(0).call())
        ];
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

        let mut sq = Sequence::new(nonempty![boxed(m1), boxed(m2), boxed(m3)]);

        assert_eq!(sq.tick(), TreeStatus::Running);
        assert_eq!(sq.tick(), TreeStatus::Running);
        assert_eq!(sq.tick(), TreeStatus::Running);
    }
}

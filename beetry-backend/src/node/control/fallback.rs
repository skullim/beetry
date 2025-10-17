use crate::{
    node::{
        Node, NodeIdentifier, NodeType, TracedNode,
        control::RunningNodesAborter,
        nonempty::{Indices, NonEmptyNodes},
    },
    node_impl,
    status::TreeStatus,
};

struct FallbackControl {
    nodes: NonEmptyNodes,
    aborter: RunningNodesAborter,
}

impl FallbackControl {
    fn new(nodes: NonEmptyNodes) -> Self {
        Self {
            nodes,
            aborter: RunningNodesAborter::new(),
        }
    }
}

impl Node for FallbackControl {
    fn tick(&mut self) -> TreeStatus {
        let aborter = &mut self.aborter;
        for idx in self.nodes.indices() {
            let node = &mut self.nodes[idx];
            match node.tick() {
                TreeStatus::Failure => {
                    aborter.untrack(idx);
                    continue;
                }
                TreeStatus::Running => {
                    aborter.abort_if_other_running(&mut self.nodes, idx);
                    aborter.track(idx);
                    return TreeStatus::Running;
                }
                TreeStatus::Success => {
                    aborter.abort_if_other_running(&mut self.nodes, idx);
                    return TreeStatus::Success;
                }
            }
        }
        TreeStatus::Failure
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

pub struct Fallback(TracedNode<FallbackControl>);
impl Fallback {
    pub fn new(nodes: impl Into<NonEmptyNodes>) -> Self {
        Self(TracedNode::new(
            FallbackControl::new(nodes.into()),
            NodeIdentifier::new(NodeType::Fallback),
        ))
    }
}

node_impl!(Fallback);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::MockNode;
    use crate::node::Node;
    use crate::node::mock_test::{boxed, mock, tick_returns};
    use crate::status::TreeStatus;

    #[test]
    fn success_with_first_success() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TreeStatus::Failure).times(1).call()),
            boxed(mock().status(TreeStatus::Success).times(1).call()),
            boxed(mock().status(TreeStatus::Success).times(0).call()),
        ]);
        let mut fb = Fallback::new(nodes);

        assert_eq!(fb.tick(), TreeStatus::Success);
    }

    #[test]
    fn running_with_first_running() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TreeStatus::Failure).times(1).call()),
            boxed(mock().status(TreeStatus::Running).times(1).call()),
            boxed(mock().status(TreeStatus::Success).times(0).call()),
        ]);
        let mut fb = Fallback::new(nodes);
        assert_eq!(fb.tick(), TreeStatus::Running);
    }

    #[test]
    fn failure_with_all_failed() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TreeStatus::Failure).times(1).call()),
            boxed(mock().status(TreeStatus::Failure).times(1).call()),
        ]);
        let mut fb = Fallback::new(nodes);

        assert_eq!(fb.tick(), TreeStatus::Failure);
    }

    #[test]
    fn resets_running() {
        let (mut m1, mut m2, mut m3) = (MockNode::new(), MockNode::new(), MockNode::new());
        tick_returns(
            &mut m1,
            vec![
                TreeStatus::Failure,
                TreeStatus::Failure,
                TreeStatus::Running,
            ],
        );
        tick_returns(&mut m2, vec![TreeStatus::Failure, TreeStatus::Running]);
        tick_returns(&mut m3, vec![TreeStatus::Running]);
        m2.expect_abort().once().return_const(());
        m3.expect_abort().once().return_const(());

        let nodes = NonEmptyNodes::from([boxed(m1), boxed(m2), boxed(m3)]);
        let mut fb = Fallback::new(nodes);

        assert_eq!(fb.tick(), TreeStatus::Running);
        assert_eq!(fb.tick(), TreeStatus::Running);
        assert_eq!(fb.tick(), TreeStatus::Running);
    }
}

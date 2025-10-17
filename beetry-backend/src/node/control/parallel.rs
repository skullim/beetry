use crate::{
    node::{
        Node,
        control::RunningNodesAborter,
        nonempty::{Indices, NonEmptyNodes},
    },
    status::TreeStatus,
};

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
    fn tick(&mut self) -> TreeStatus {
        let aborter: &mut RunningNodesAborter = &mut self.aborter;
        for idx in self.nodes.indices() {
            let node = &mut self.nodes[idx];
            match node.tick() {
                TreeStatus::Success => {
                    aborter.untrack(idx);
                    continue;
                }
                TreeStatus::Running => {
                    aborter.track(idx);
                    continue;
                }
                TreeStatus::Failure => {
                    aborter.untrack(idx);
                    aborter.abort_all(&mut self.nodes);
                    return TreeStatus::Failure;
                }
            }
        }

        match aborter.is_any_tracked() {
            true => TreeStatus::Running,
            false => TreeStatus::Success,
        }
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
    use crate::node::MockNode;
    use crate::node::Node;
    use crate::node::mock_test::{boxed, mock, tick_returns};
    use crate::status::TreeStatus;

    #[test]
    fn success_with_all_success() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TreeStatus::Success).times(1).call()),
            boxed(mock().status(TreeStatus::Success).times(1).call()),
        ]);
        let mut pl = Parallel::new(nodes);

        assert_eq!(pl.tick(), TreeStatus::Success);
    }

    #[test]
    fn running_with_any_running() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TreeStatus::Success).times(1).call()),
            boxed(mock().status(TreeStatus::Running).times(1).call()),
            boxed(mock().status(TreeStatus::Running).times(1).call()),
        ]);
        let mut pl = Parallel::new(nodes);
        assert_eq!(pl.tick(), TreeStatus::Running);
    }

    #[test]
    fn failure_with_any_failed() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TreeStatus::Success).times(1).call()),
            boxed(mock().status(TreeStatus::Failure).times(1).call()),
            boxed(mock().status(TreeStatus::Success).times(0).call()),
        ]);
        let mut pl = Parallel::new(nodes);

        assert_eq!(pl.tick(), TreeStatus::Failure);
    }

    #[test]
    fn resets_running() {
        let (mut m1, mut m2, mut m3) = (MockNode::new(), MockNode::new(), MockNode::new());
        tick_returns(
            &mut m1,
            vec![
                TreeStatus::Running,
                TreeStatus::Running,
                TreeStatus::Failure,
            ],
        );
        tick_returns(&mut m2, vec![TreeStatus::Running, TreeStatus::Running]);
        tick_returns(&mut m3, vec![TreeStatus::Running, TreeStatus::Running]);

        m2.expect_abort().once().return_const(());
        m3.expect_abort().once().return_const(());

        let nodes = NonEmptyNodes::from([boxed(m1), boxed(m2), boxed(m3)]);
        let mut pl = Parallel::new(nodes);

        assert_eq!(pl.tick(), TreeStatus::Running);
        assert_eq!(pl.tick(), TreeStatus::Running);
        assert_eq!(pl.tick(), TreeStatus::Failure);
    }
}

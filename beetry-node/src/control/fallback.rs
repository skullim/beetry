use crate::{Indices, control::RunningNodesAborter};
use beetry_core::{ControlNode, Node, NonEmptyNodes, TickStatus};

pub struct Fallback {
    nodes: NonEmptyNodes,
    aborter: RunningNodesAborter,
}

impl Fallback {
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
                    continue;
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

impl ControlNode for Fallback {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_test::{boxed, mock, tick_returns};
    use beetry_core::{MockNode, Node, TickStatus};

    #[test]
    fn success_with_first_success() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TickStatus::Failure).times(1).call()),
            boxed(mock().status(TickStatus::Success).times(1).call()),
            boxed(mock().status(TickStatus::Success).times(0).call()),
        ]);
        let mut fb = Fallback::new(nodes);

        assert_eq!(fb.tick(), TickStatus::Success);
    }

    #[test]
    fn running_with_first_running() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TickStatus::Failure).times(1).call()),
            boxed(mock().status(TickStatus::Running).times(1).call()),
            boxed(mock().status(TickStatus::Success).times(0).call()),
        ]);
        let mut fb = Fallback::new(nodes);
        assert_eq!(fb.tick(), TickStatus::Running);
    }

    #[test]
    fn failure_with_all_failed() {
        let nodes = NonEmptyNodes::from([
            boxed(mock().status(TickStatus::Failure).times(1).call()),
            boxed(mock().status(TickStatus::Failure).times(1).call()),
        ]);
        let mut fb = Fallback::new(nodes);

        assert_eq!(fb.tick(), TickStatus::Failure);
    }

    #[test]
    fn resets_running() {
        let (mut m1, mut m2, mut m3) = (MockNode::new(), MockNode::new(), MockNode::new());
        tick_returns(
            &mut m1,
            vec![
                TickStatus::Failure,
                TickStatus::Failure,
                TickStatus::Running,
            ],
        );
        tick_returns(&mut m2, vec![TickStatus::Failure, TickStatus::Running]);
        tick_returns(&mut m3, vec![TickStatus::Running]);
        m2.expect_abort().once().return_const(());
        m3.expect_abort().once().return_const(());

        let nodes = NonEmptyNodes::from([boxed(m1), boxed(m2), boxed(m3)]);
        let mut fb = Fallback::new(nodes);

        assert_eq!(fb.tick(), TickStatus::Running);
        assert_eq!(fb.tick(), TickStatus::Running);
        assert_eq!(fb.tick(), TickStatus::Running);
    }
}

use beetry_core::{Node, TickStatus};

pub struct Invert<N> {
    node: N,
}

impl<N> Invert<N> {
    pub fn new(node: N) -> Self {
        Self { node }
    }
}

impl<N> Node for Invert<N>
where
    N: Node,
{
    fn tick(&mut self) -> TickStatus {
        match self.node.tick() {
            TickStatus::Success => TickStatus::Failure,
            TickStatus::Failure => TickStatus::Success,
            TickStatus::Running => TickStatus::Running,
        }
    }

    fn abort(&mut self) {
        self.node.abort();
    }

    fn reset(&mut self) {
        self.node.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_test::mock;
    use beetry_core::{MockNode, Node, TickStatus};

    #[test]
    fn success_becomes_failure() {
        let mut invert = Invert::new(mock().status(TickStatus::Success).times(1).call());
        assert_eq!(invert.tick(), TickStatus::Failure);
    }

    #[test]
    fn failure_becomes_success() {
        let mut invert = Invert::new(mock().status(TickStatus::Failure).times(1).call());
        assert_eq!(invert.tick(), TickStatus::Success);
    }

    #[test]
    fn running_stays_running() {
        let mut invert = Invert::new(mock().status(TickStatus::Running).times(1).call());
        assert_eq!(invert.tick(), TickStatus::Running);
    }

    #[test]
    fn abort_is_propagated() {
        let mut node = MockNode::new();
        node.expect_abort().once().return_const(());
        let mut invert = Invert::new(node);
        invert.abort();
    }

    #[test]
    fn reset_is_propagated() {
        let mut node = MockNode::new();
        node.expect_reset().once().return_const(());
        let mut invert = Invert::new(node);
        invert.reset();
    }
}

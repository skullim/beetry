use beetry_core::{Node, TickStatus};

/// Keeps returning [`TickStatus::Running`] until the child succeeds.
pub struct UntilSuccess<N> {
    node: N,
}

impl<N> UntilSuccess<N> {
    #[must_use]
    pub fn new(node: N) -> Self {
        Self { node }
    }
}

impl<N> Node for UntilSuccess<N>
where
    N: Node,
{
    fn tick(&mut self) -> TickStatus {
        match self.node.tick() {
            TickStatus::Success => TickStatus::Success,
            TickStatus::Failure | TickStatus::Running => TickStatus::Running,
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
    use crate::mock_test::mock_returns;
    use beetry_core::{Node, TickStatus};

    #[test]
    fn success_stays_success() {
        let mut node = UntilSuccess::new(mock_returns([TickStatus::Success]));
        assert_eq!(node.tick(), TickStatus::Success);
    }

    #[test]
    fn failure_becomes_running() {
        let mut node = UntilSuccess::new(mock_returns([TickStatus::Failure]));
        assert_eq!(node.tick(), TickStatus::Running);
    }

    #[test]
    fn running_stays_running() {
        let mut node = UntilSuccess::new(mock_returns([TickStatus::Running]));
        assert_eq!(node.tick(), TickStatus::Running);
    }

    #[test]
    fn abort_is_propagated() {
        let mut child = mock_returns([]);
        child.expect_abort().once().return_const(());
        let mut node = UntilSuccess::new(child);
        node.abort();
    }

    #[test]
    fn reset_is_propagated() {
        let mut child = mock_returns([]);
        child.expect_reset().once().return_const(());
        let mut node = UntilSuccess::new(child);
        node.reset();
    }
}

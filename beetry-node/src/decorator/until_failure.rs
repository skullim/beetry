use beetry_core::{Node, TickStatus};

/// Keeps returning [`TickStatus::Running`] until the child fails.
pub struct UntilFailure<N> {
    node: N,
}

impl<N> UntilFailure<N> {
    #[must_use]
    pub fn new(node: N) -> Self {
        Self { node }
    }
}

impl<N> Node for UntilFailure<N>
where
    N: Node,
{
    fn tick(&mut self) -> TickStatus {
        match self.node.tick() {
            TickStatus::Failure => TickStatus::Failure,
            TickStatus::Success | TickStatus::Running => TickStatus::Running,
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
    use beetry_core::{Node, TickStatus};

    use super::*;
    use crate::mock_test::mock_returns;

    #[test]
    fn failure_stays_failure() {
        let mut node = UntilFailure::new(mock_returns([TickStatus::Failure]));
        assert_eq!(node.tick(), TickStatus::Failure);
    }

    #[test]
    fn success_becomes_running() {
        let mut node = UntilFailure::new(mock_returns([TickStatus::Success]));
        assert_eq!(node.tick(), TickStatus::Running);
    }

    #[test]
    fn running_stays_running() {
        let mut node = UntilFailure::new(mock_returns([TickStatus::Running]));
        assert_eq!(node.tick(), TickStatus::Running);
    }

    #[test]
    fn abort_is_propagated() {
        let mut child = mock_returns([]);
        child.expect_abort().once().return_const(());
        let mut node = UntilFailure::new(child);
        node.abort();
    }

    #[test]
    fn reset_is_propagated() {
        let mut child = mock_returns([]);
        child.expect_reset().once().return_const(());
        let mut node = UntilFailure::new(child);
        node.reset();
    }
}

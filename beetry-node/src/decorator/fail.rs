use beetry_core::{Node, TickStatus};

/// Forces any terminal child result to become [`TickStatus::Failure`].
pub struct Fail<N> {
    node: N,
}

impl<N> Fail<N> {
    #[must_use]
    pub fn new(node: N) -> Self {
        Self { node }
    }
}

impl<N> Node for Fail<N>
where
    N: Node,
{
    fn tick(&mut self) -> TickStatus {
        match self.node.tick() {
            TickStatus::Running => TickStatus::Running,
            TickStatus::Success | TickStatus::Failure => TickStatus::Failure,
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
    fn success_becomes_failure() {
        let mut fail = Fail::new(mock_returns([TickStatus::Success]));
        assert_eq!(fail.tick(), TickStatus::Failure);
    }

    #[test]
    fn failure_stays_failure() {
        let mut fail = Fail::new(mock_returns([TickStatus::Failure]));
        assert_eq!(fail.tick(), TickStatus::Failure);
    }

    #[test]
    fn running_stays_running() {
        let mut fail = Fail::new(mock_returns([TickStatus::Running]));
        assert_eq!(fail.tick(), TickStatus::Running);
    }

    #[test]
    fn abort_is_propagated() {
        let mut node = mock_returns([]);
        node.expect_abort().once().return_const(());
        let mut fail = Fail::new(node);
        fail.abort();
    }

    #[test]
    fn reset_is_propagated() {
        let mut node = mock_returns([]);
        node.expect_reset().once().return_const(());
        let mut fail = Fail::new(node);
        fail.reset();
    }
}

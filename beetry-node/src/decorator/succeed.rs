use beetry_core::{Node, TickStatus};

/// Forces any terminal child result to become [`TickStatus::Success`].
pub struct Succeed<N> {
    node: N,
}

impl<N> Succeed<N> {
    #[must_use]
    pub fn new(node: N) -> Self {
        Self { node }
    }
}

impl<N> Node for Succeed<N>
where
    N: Node,
{
    fn tick(&mut self) -> TickStatus {
        match self.node.tick() {
            TickStatus::Running => TickStatus::Running,
            TickStatus::Success | TickStatus::Failure => TickStatus::Success,
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
    fn success_stays_success() {
        let mut succeed = Succeed::new(mock_returns([TickStatus::Success]));
        assert_eq!(succeed.tick(), TickStatus::Success);
    }

    #[test]
    fn failure_becomes_success() {
        let mut succeed = Succeed::new(mock_returns([TickStatus::Failure]));
        assert_eq!(succeed.tick(), TickStatus::Success);
    }

    #[test]
    fn running_stays_running() {
        let mut succeed = Succeed::new(mock_returns([TickStatus::Running]));
        assert_eq!(succeed.tick(), TickStatus::Running);
    }

    #[test]
    fn abort_is_propagated() {
        let mut node = mock_returns([]);
        node.expect_abort().once().return_const(());
        let mut succeed = Succeed::new(node);
        succeed.abort();
    }

    #[test]
    fn reset_is_propagated() {
        let mut node = mock_returns([]);
        node.expect_reset().once().return_const(());
        let mut succeed = Succeed::new(node);
        succeed.reset();
    }
}

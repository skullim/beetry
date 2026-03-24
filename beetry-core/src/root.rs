use crate::{Node, TickStatus};

/// Root node.
///
/// `Root` owns the top-level child node and forwards the [`Node`] lifecycle to
/// it.
pub struct Root<N> {
    child: N,
}

impl<N> Root<N>
where
    N: Node,
{
    pub fn new(child: N) -> Self {
        Self { child }
    }
}

impl<N> Node for Root<N>
where
    N: Node,
{
    fn tick(&mut self) -> TickStatus {
        self.child.tick()
    }

    fn reset(&mut self) {
        self.child.reset();
    }

    fn abort(&mut self) {
        self.child.abort();
    }
}

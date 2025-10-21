use crate::{Node, TreeStatus};

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
    fn tick(&mut self) -> TreeStatus {
        self.child.tick()
    }

    fn reset(&mut self) {
        self.child.reset();
    }

    fn abort(&mut self) {
        self.child.abort();
    }
}

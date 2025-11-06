#[cfg(test)]
pub mod test {
    use beetry_core::{MockNode, Node, TickStatus};
    use bon::builder;

    pub fn boxed<N: Node + 'static>(n: N) -> Box<dyn Node> {
        Box::new(n)
    }

    #[builder]
    pub fn mock(status: TickStatus, times: usize) -> MockNode {
        let mut m = MockNode::new();
        m.expect_tick().return_const(status).times(times);
        m
    }

    pub fn tick_returns(m: &mut MockNode, statuses: Vec<TickStatus>) {
        let mut it = statuses.into_iter();
        m.expect_tick().returning(move || it.next().unwrap());
    }
}

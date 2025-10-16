#[cfg(test)]
pub(crate) mod test {
    use bon::builder;

    use crate::node::MockNode;
    use crate::node::Node;
    use crate::status::TreeStatus;

    pub(crate) fn boxed<N: Node + 'static>(n: N) -> Box<dyn Node> {
        Box::new(n)
    }

    #[builder]
    pub(crate) fn mock(status: TreeStatus, times: usize) -> MockNode {
        let mut m = MockNode::new();
        m.expect_tick().return_const(status).times(times);
        m
    }

    pub(crate) fn tick_returns(m: &mut MockNode, statuses: Vec<TreeStatus>) {
        let mut it = statuses.into_iter();
        m.expect_tick().returning(move || it.next().unwrap());
    }
}

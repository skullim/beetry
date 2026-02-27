#[cfg(test)]
pub mod test {
    use beetry_core::{MockNode, Node, TickStatus};

    pub fn boxed<N: Node + 'static>(n: N) -> Box<dyn Node> {
        Box::new(n)
    }

    pub fn mock_returns<I>(statuses: I) -> MockNode
    where
        I: IntoIterator<Item = TickStatus>,
    {
        let mut m = MockNode::new();
        tick_returns(&mut m, statuses);
        m
    }

    fn tick_returns<I>(m: &mut MockNode, statuses: I)
    where
        I: IntoIterator<Item = TickStatus>,
    {
        let mut it = statuses.into_iter().collect::<Vec<_>>().into_iter();
        m.expect_tick().returning(move || it.next().unwrap());
    }
}

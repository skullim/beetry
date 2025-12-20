use num_traits::One;
use std::{hash::Hash, ops::AddAssign};

#[derive(Debug, Default)]
pub struct IdProvider<I> {
    id: I,
}

impl<I> IdProvider<I>
where
    I: Default + Copy + AddAssign + Hash + One,
{
    pub fn next_available_id(&mut self, find_predicate: impl Fn(&I) -> bool) -> I {
        std::iter::repeat_with(|| self.next())
            .find(find_predicate)
            .expect("Provider is infinite")
    }

    fn next(&mut self) -> I {
        let id = self.id;
        // simple monotonically increasing id increment. There might be holes if elements are deleted, but accept the tradeoff for now
        self.id += I::one();
        id
    }
}

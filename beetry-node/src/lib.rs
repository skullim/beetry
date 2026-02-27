mod control;
mod decorator;

#[cfg(feature = "registry")]
mod registry;

use beetry_core::NonEmptyNodes;
pub use control::{Fallback, MemSequence, Parallel, Sequence};
pub use decorator::{Invert, Succeed, UntilFailure, UntilSuccess};

#[cfg(test)]
mod mock;
#[cfg(test)]
pub(crate) use mock::test as mock_test;

pub(crate) trait Indices {
    fn indices(&self) -> std::ops::Range<usize>;
}

impl Indices for NonEmptyNodes {
    fn indices(&self) -> std::ops::Range<usize> {
        0..self.len().into()
    }
}

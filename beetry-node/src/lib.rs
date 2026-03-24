//! # Beetry Node
//!
//! `beetry-node` provides the built-in control and decorator behavior tree
//! nodes available by default in Beetry.
//!
//! Compatibility with the editor integration and plugin-based registration is
//! guarded behind the `plugin` feature.

mod control;
mod decorator;

#[cfg(feature = "plugin")]
pub mod plugin;

use beetry_core::NonEmptyNodes;
pub use control::{Fallback, MemorySequence, Parallel, ParallelParams, Sequence};
pub use decorator::{Fail, Invert, Succeed, UntilFailure, UntilSuccess};

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

//! Shared node trait and built-in non-leaf node types.

pub use beetry_core::{BoxNode, Node, NonEmptyNodes, Root};
pub use beetry_node::{
    Fail, Fallback, Invert, MemSequence, Parallel, ParallelParams, Sequence, Succeed, UntilFailure,
    UntilSuccess,
};

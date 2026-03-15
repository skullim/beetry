mod fallback;
mod parallel;
mod sequence;

use std::collections::BTreeSet;

use beetry_core::BoxNode;
pub use fallback::Fallback;
pub use parallel::{Parallel, ParallelParams};
pub use sequence::{MemSequence, Sequence};

struct RunningNodesAborter {
    running: BTreeSet<usize>,
}

impl RunningNodesAborter {
    fn new() -> Self {
        Self {
            running: BTreeSet::new(),
        }
    }

    fn is_any_tracked(&self) -> bool {
        !self.running.is_empty()
    }

    fn track(&mut self, idx: usize) {
        self.running.insert(idx);
    }

    fn clear(&mut self) {
        self.running.clear();
    }

    fn untrack(&mut self, idx: usize) {
        self.running.take(&idx);
    }

    fn abort_if_other_running(&mut self, nodes: &mut [BoxNode], idx: usize) {
        if !self.running.contains(&idx) {
            self.abort_all(nodes);
        }
    }

    fn abort_all(&mut self, nodes: &mut [BoxNode]) {
        while let Some(idx) = self.running.pop_first() {
            nodes[idx].abort();
        }
    }
}

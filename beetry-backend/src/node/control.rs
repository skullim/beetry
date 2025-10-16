mod fallback;
mod parallel;
mod sequence;

use std::collections::BTreeSet;

pub use fallback::Fallback;
pub use parallel::Parallel;
pub use sequence::Sequence;

use crate::Node;
use nonempty::NonEmpty;

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

    fn abort_if_other_running(&mut self, nodes: &mut NonEmpty<Box<dyn Node>>, other: usize) {
        if !self.running.contains(&other) {
            self.abort_all(nodes);
        }
    }

    fn abort_all(&mut self, nodes: &mut NonEmpty<Box<dyn Node>>) {
        while let Some(idx) = self.running.pop_first() {
            nodes[idx].abort();
        }
    }
}

mod builder;
mod engine;
mod ticker;

pub use builder::Builder;
pub use engine::TreeEngine;
pub use ticker::Ticker;

use crate::{
    node::{BoxedNode, Node, NodeIdentifier, NodeType},
    status::TreeStatus,
};
use tracing::instrument;

pub struct BehaviorTree {
    root: BoxedNode,
    id: NodeIdentifier,
}

impl BehaviorTree {
    pub fn new(root: BoxedNode) -> Self {
        Self {
            root,
            id: NodeIdentifier::new(NodeType::Root),
        }
    }
}

impl Node for BehaviorTree {
    #[instrument(skip_all, fields(id=%self.id))]
    fn reset(&mut self) {
        self.root.reset();
    }

    #[instrument(skip_all, fields(id=%self.id))]
    fn tick(&mut self) -> TreeStatus {
        self.root.tick()
    }
}

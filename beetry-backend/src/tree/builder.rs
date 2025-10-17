use crate::{
    ActionBehavior, BehaviorTree, Node, Parallel, Sequence,
    node::{Action, Condition, ConditionBehavior, NonEmptyNodes, Root},
    task::Registry,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Builder {
    registry: Arc<Registry>,
}

impl Builder {
    pub(crate) fn new(registry: Arc<Registry>) -> Self {
        Self { registry }
    }

    pub fn action(&self, behavior: impl ActionBehavior + 'static) -> Box<dyn Node> {
        Box::new(Action::new(behavior, Arc::clone(&self.registry)))
    }

    pub fn condition(&self, behavior: impl ConditionBehavior + 'static) -> Box<dyn Node> {
        Box::new(Condition::new(behavior))
    }

    pub fn parallel(&self, nodes: impl Into<NonEmptyNodes>) -> Box<dyn Node> {
        Box::new(Parallel::new(nodes))
    }

    pub fn sequence(&self, nodes: impl Into<NonEmptyNodes>) -> Box<dyn Node> {
        Box::new(Sequence::new(nodes))
    }

    pub fn tree<N>(&self, root: Root<N>) -> BehaviorTree<N>
    where
        N: Node,
    {
        BehaviorTree::new(root)
    }
}

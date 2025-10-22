use beetry_core::{
    Action, ActionBehavior, Tree, Condition, ConditionBehavior, Node, RegisterTask, Root,
    TaskControl,
};
use beetry_node::{NonEmptyNodes, Parallel, Sequence};
use std::{marker::PhantomData, sync::Arc};

#[derive(Clone)]
pub struct Builder<R, T> {
    registry: Arc<R>,
    _phantom: PhantomData<T>,
}

impl<R, T> Builder<R, T>
where
    R: RegisterTask<T> + 'static,
    T: TaskControl + 'static,
{
    pub fn new(registry: R) -> Self {
        Self {
            registry: Arc::new(registry),
            _phantom: PhantomData,
        }
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

    pub fn tree<N>(&self, root: Root<N>) -> Tree<N>
    where
        N: Node,
    {
        Tree::new(root)
    }
}

use std::{marker::PhantomData, sync::Arc};

use crate::{
    Action, ActionBehavior, BoxNode, Condition, ConditionBehavior, RegisterTask, TaskHandle,
};

#[derive(Clone)]
pub struct Builder<R, T> {
    registry: Arc<R>,
    _phantom: PhantomData<T>,
}

impl<R, T> Builder<R, T>
where
    R: RegisterTask<T> + 'static,
    T: TaskHandle + 'static,
{
    pub fn new(registry: R) -> Self {
        Self {
            registry: Arc::new(registry),
            _phantom: PhantomData,
        }
    }

    pub fn action(
        &self,
        behavior: impl ActionBehavior + 'static,
    ) -> Action<R, T, impl ActionBehavior> {
        Action::new(behavior, Arc::clone(&self.registry))
    }

    pub fn action_box(&self, behavior: impl ActionBehavior + 'static) -> BoxNode {
        Box::new(Action::new(behavior, Arc::clone(&self.registry)))
    }

    pub fn condition_box(&self, behavior: impl ConditionBehavior + 'static) -> BoxNode {
        Box::new(Condition::new(behavior))
    }
}

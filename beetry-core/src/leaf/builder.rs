use crate::{
    ActionBehavior, BoxNode, ConditionBehavior, RegisterTask, TaskHandle,
};
use std::marker::PhantomData;
use std::sync::Arc;

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

    pub fn action(&self, behavior: impl ActionBehavior + 'static) -> BoxNode {
        Box::new(crate::Action::new(behavior, Arc::clone(&self.registry)))
    }

    pub fn condition(&self, behavior: impl ConditionBehavior + 'static) -> BoxNode {
        Box::new(crate::Condition::new(behavior))
    }
}

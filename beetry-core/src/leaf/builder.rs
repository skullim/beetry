use std::{marker::PhantomData, sync::Arc, time::Duration};

use crate::{
    Action, ActionBehavior, BoxNode, Condition, ConditionBehavior, RegisterTask, TaskHandle,
};

#[derive(Clone)]
pub struct Builder<R, T> {
    registry: Arc<R>,
    abort_poll_interval: Duration,
    _phantom: PhantomData<T>,
}

impl<R, T> Builder<R, T>
where
    R: RegisterTask<T> + 'static,
    T: TaskHandle + 'static,
{
    pub fn new(registry: R, abort_poll_interval: Duration) -> Self {
        Self {
            registry: Arc::new(registry),
            abort_poll_interval,
            _phantom: PhantomData,
        }
    }

    pub fn action<B>(&self, behavior: B) -> Action<R, T, B>
    where
        B: ActionBehavior + 'static,
    {
        Action::new(
            behavior,
            Arc::clone(&self.registry),
            self.abort_poll_interval,
        )
    }

    pub fn action_box(&self, behavior: impl ActionBehavior + 'static) -> BoxNode {
        Box::new(Action::new(
            behavior,
            Arc::clone(&self.registry),
            self.abort_poll_interval,
        ))
    }

    pub fn condition_box(&self, behavior: impl ConditionBehavior + 'static) -> BoxNode {
        Box::new(Condition::new(behavior))
    }
}

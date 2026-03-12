use crate::{Node, TickStatus};

pub trait Behavior {
    /// Evaluate the condition for the current tick.
    ///
    /// Returning `true` maps to [`TickStatus::Success`] and `false` maps to
    /// [`TickStatus::Failure`] in [`Condition`].
    fn cond(&mut self) -> bool;

    /// Reset any condition-local state for a fresh run.
    fn reset(&mut self) {}
}

pub type BoxBehavior = Box<dyn Behavior>;
impl Behavior for BoxBehavior {
    fn cond(&mut self) -> bool {
        (**self).cond()
    }
    fn reset(&mut self) {
        (**self).reset();
    }
}

pub struct Condition<B>
where
    B: Behavior,
{
    behavior: B,
}

impl<B> Condition<B>
where
    B: Behavior,
{
    pub fn new(behavior: B) -> Self {
        Self { behavior }
    }
}

impl<B> Node for Condition<B>
where
    B: Behavior,
{
    fn tick(&mut self) -> TickStatus {
        if self.behavior.cond() {
            TickStatus::Success
        } else {
            TickStatus::Failure
        }
    }
    fn reset(&mut self) {
        self.behavior.reset();
    }
}

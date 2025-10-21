use crate::{Node, TreeStatus};

pub trait Behavior {
    fn cond(&mut self) -> bool;
    fn reset(&mut self) {}
}

impl Behavior for Box<dyn Behavior> {
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
    fn tick(&mut self) -> TreeStatus {
        match self.behavior.cond() {
            true => TreeStatus::Success,
            false => TreeStatus::Failure,
        }
    }
    fn reset(&mut self) {
        self.behavior.reset();
    }
}

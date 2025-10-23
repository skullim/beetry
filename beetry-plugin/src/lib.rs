pub mod channel;
pub mod node;

pub trait Plugin: Send + Sync {
    type Spec;
    type Factory;

    fn new() -> Self
    where
        Self: Sized;

    fn spec(&self) -> Self::Spec;

    fn factory(self: Box<Self>) -> Self::Factory;
}

pub mod channel;
pub mod node;

pub trait Plugin {
    type Spec;
    type Factory;

    fn new() -> Self
    where
        Self: Sized;

    fn spec(&self) -> Self::Spec;

    fn factory(self: Box<Self>) -> Self::Factory;
}

pub type BoxPlugin<S, F> = Box<dyn Plugin<Spec = S, Factory = F>>;

pub trait Named {
    fn name(&self) -> &str;
}

pub trait ConstructPlugin {
    type Spec: Named;
    type Factory;
    fn construct(&self) -> BoxPlugin<Self::Spec, Self::Factory>;
}

pub use inventory;

#[macro_export]
macro_rules! submit {
    ($plugin:expr) => {
        $crate::inventory::submit!($plugin);
    };
}

// reexport for macro
pub use beetry_serde::ser::node::ActionSpec;

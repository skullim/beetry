use crate::{BoxPlugin, ConstructPlugin, Named, PluginConstructor, PluginError, unique_plugins};
use anyhow::Result;
use beetry_channel::{AnyBoxReceiver, AnyBoxSender};
use beetry_core::{BoxActionBehavior, BoxConditionBehavior, BoxNode, NonEmptyNodes};
use beetry_plugin_types::node::{ActionSpec, ConditionSpec, ControlSpec, NodeSpec};
use beetry_reconstruction_types::parameter::Parameters;
use bon::Builder;
use std::marker::PhantomData;

//@todo move those types to beetry-reconstruction-types crate
pub type LeafReconstructionData = NodeReconstructionData<LeafMetadata>;
pub type ActionReconstructionData = LeafReconstructionData;
pub type ConditionReconstructionData = LeafReconstructionData;
pub type ControlReconstructionData = NodeReconstructionData<ControlMetadata>;

#[derive(Builder)]
pub struct NodeReconstructionData<D> {
    pub inner: D,
    #[builder(default)]
    pub parameters: Parameters,
}

pub enum NodeMetadata {
    Leaf(LeafMetadata),
    Control(ControlMetadata),
}

#[derive(Default, Builder)]
pub struct LeafMetadata {
    #[builder(default, into)]
    pub receivers: Vec<AnyBoxReceiver>,
    #[builder(default, into)]
    pub senders: Vec<AnyBoxSender>,
}

impl LeafMetadata {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct ControlMetadata {
    pub children: NonEmptyNodes,
}

impl ControlMetadata {
    pub fn new(children: NonEmptyNodes) -> Self {
        Self { children }
    }
}

pub type ActionFactory = Factory<BoxActionFactoryFn, ActionReconstructionData, BoxActionBehavior>;
type BoxActionFactoryFn = Box<dyn Fn(ActionReconstructionData) -> Result<BoxActionBehavior>>;

pub type ConditionFactory =
    Factory<BoxConditionFactoryFn, ConditionReconstructionData, BoxConditionBehavior>;
type BoxConditionFactoryFn =
    Box<dyn Fn(ConditionReconstructionData) -> Result<BoxConditionBehavior>>;

pub type ControlFactory = Factory<BoxControlFactoryFn, ControlReconstructionData, BoxNode>;
type BoxControlFactoryFn = Box<dyn Fn(ControlReconstructionData) -> Result<BoxNode>>;

pub struct Factory<F, I, O> {
    func: F,
    _ph1: PhantomData<I>,
    _ph2: PhantomData<O>,
}

impl<F, I, O> Factory<F, I, O>
// I: Input
// O: Output
where
    F: Fn(I) -> Result<O>,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _ph1: PhantomData,
            _ph2: PhantomData,
        }
    }

    pub fn try_create(&self, data: I) -> Result<O> {
        (self.func)(data)
    }
}

pub type BoxActionPlugin = BoxPlugin<ActionSpec, ActionFactory>;
pub type BoxConditionPlugin = BoxPlugin<ConditionSpec, ConditionFactory>;
pub type BoxControlPlugin = BoxPlugin<ControlSpec, ControlFactory>;

impl<S> Named for NodeSpec<S> {
    fn name(&self) -> &str {
        self.name.0.as_str()
    }
}

pub type ActionPluginConstructor = PluginConstructor<ActionSpec, ActionFactory>;
pub type ConditionPluginConstructor = PluginConstructor<ConditionSpec, ConditionFactory>;
pub type ControlPluginConstructor = PluginConstructor<ControlSpec, ControlFactory>;

impl ActionPluginConstructor {
    pub fn plugins() -> Result<Vec<BoxActionPlugin>, PluginError> {
        unique_plugins::<Self, <Self as ConstructPlugin>::Spec, <Self as ConstructPlugin>::Factory>(
        )
    }
}

impl ConditionPluginConstructor {
    pub fn plugins() -> Result<Vec<BoxConditionPlugin>, PluginError> {
        unique_plugins::<Self, <Self as ConstructPlugin>::Spec, <Self as ConstructPlugin>::Factory>(
        )
    }
}

impl ControlPluginConstructor {
    pub fn plugins() -> Result<Vec<BoxControlPlugin>, PluginError> {
        unique_plugins::<Self, <Self as ConstructPlugin>::Spec, <Self as ConstructPlugin>::Factory>(
        )
    }
}

inventory::collect! {ActionPluginConstructor}
inventory::collect! {ConditionPluginConstructor}
inventory::collect! {ControlPluginConstructor}

#[macro_export]
macro_rules! plugin {
    ($plugin_name:ident : Action {
        spec = $spec:expr,
        factory_fn = $factory_fn:expr $(,)?
    }) => {
        $crate::plugin_impl!(
            plugin = $plugin_name,
            spec_ty = $crate::ActionSpec,
            factory_ty = $crate::node::ActionFactory,
            spec = $spec,
            factory_fn = $factory_fn
        );
    };

    ($plugin_name:ident : Condition {
        spec = $spec:expr,
        factory_fn = $factory_fn:expr $(,)?
    }) => {
        $crate::plugin_impl!(
            plugin = $plugin_name,
            spec_ty = $crate::ConditionSpec,
            factory_ty = $crate::node::ConditionFactory,
            spec = $spec,
            factory_fn = $factory_fn
        );
    };
}

#[macro_export]
macro_rules! plugin_impl {
    (   plugin = $plugin_name:ident,
        spec_ty = $spec_ty:ty,
        factory_ty = $factory_ty:ty,
        spec = $spec:expr,
        factory_fn = $factory_fn:expr
    ) => {
        pub struct $plugin_name;

        impl $crate::Plugin for $plugin_name {
            type Spec = $spec_ty;
            type Factory = $factory_ty;

            fn new() -> Self
            where
                Self: Sized,
            {
                Self
            }

            fn spec(&self) -> Self::Spec {
                $spec
            }

            fn factory(self: Box<Self>) -> Self::Factory {
                Self::Factory::new(Box::new($factory_fn))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::Plugin;

    use super::*;
    use beetry_plugin_types::node::{ActionLeafSchema, NodeName};

    struct TestPluginA;

    impl Plugin for TestPluginA {
        type Spec = ActionSpec;
        type Factory = ActionFactory;

        fn new() -> Self {
            Self
        }

        fn spec(&self) -> Self::Spec {
            ActionSpec::builder()
                .name(NodeName::new("TestPlugin"))
                .schema(ActionLeafSchema::default())
                .build()
        }

        fn factory(self: Box<Self>) -> Self::Factory {
            ActionFactory::new(Box::new(|_| {
                Err(anyhow::anyhow!("This is a test factory, not functional"))
            }))
        }
    }

    struct TestPluginB;

    impl Plugin for TestPluginB {
        type Spec = ActionSpec;
        type Factory = ActionFactory;

        fn new() -> Self {
            Self
        }

        fn spec(&self) -> Self::Spec {
            ActionSpec::builder()
                .name(NodeName::new("TestPlugin"))
                .schema(ActionLeafSchema::default())
                .build()
        }

        fn factory(self: Box<Self>) -> Self::Factory {
            ActionFactory::new(Box::new(|_| {
                Err(anyhow::anyhow!("This is a test factory, not functional"))
            }))
        }
    }

    inventory::submit! {
        ActionPluginConstructor::new::<TestPluginA>()
    }

    //@todo registering duplicated entry might affect other tests when plugins() method is called.
    //Better to avoid global registration if possible
    inventory::submit! {
        ActionPluginConstructor::new::<TestPluginB>()
    }

    #[test]
    fn test_duplicate_plugin_name_error() {
        let result = ActionPluginConstructor::plugins();
        assert!(matches!(
            result,
            Err(PluginError::DuplicateName(name)) if name == NodeName::new("TestPlugin").0
        ));
    }
}

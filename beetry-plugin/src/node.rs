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

//@todo improve to factor out hard coded types
#[macro_export]
macro_rules! plugin2 {
    ($plugin_name:ident : Action {
        name: $name:literal,
        receivers: {
            $(
                $receiver:ty => $desc:literal
            ),* $(,)?
        }
    }) => {
        pub struct $plugin_name {
            spec: $crate::ActionSpec,
            factory: $crate::node::ActionFactory,
        }

        impl $crate::Plugin for $plugin_name {
            type Spec = $crate::ActionSpec;
            type Factory = $crate::node::ActionFactory;

            fn new() -> Self
            where
                Self: Sized,
            {
                let factory_fn = |mut data: ActionReconstructionData| {
                    let receivers = downcast! {receivers = &mut data.inner.receivers, expected = [ $($receiver),*]}
                    .map_err(|_| anyhow!("failed to obtain typed receivers"))?;
                    Ok(Box::new(Drive::new(
                        DriveReceivers::builder().pose(receivers.0).build())) as BoxActionBehavior)
                };
                let spec = $crate::ActionSpec::builder().name($name).schema(beetry_plugin_types::schema!{kind = action, receivers = [$($receiver, desc = $desc),* ]}).build();

                Self {
                    spec,
                    factory: Self::Factory::new(Box::new(factory_fn)),
                }
            }

            fn spec(&self) -> &Self::Spec {
                &self.spec
            }

            fn factory(&self) -> &Self::Factory {
                &self.factory
            }

            fn into_parts(self: Box<Self>) -> (Self::Spec, Self::Factory) {
                (self.spec, self.factory)
            }
        }
    };
}

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
        pub struct $plugin_name {
            spec: $spec_ty,
            factory: $factory_ty,
        }

        impl $crate::Plugin for $plugin_name {
            type Spec = $spec_ty;
            type Factory = $factory_ty;

            fn new() -> Self
            where
                Self: Sized,
            {
                Self {
                    spec: $spec,
                    factory: Self::Factory::new(Box::new($factory_fn)),
                }
            }

            fn spec(&self) -> &Self::Spec {
                &self.spec
            }

            fn factory(&self) -> &Self::Factory {
                &self.factory
            }

            fn into_parts(self: Box<Self>) -> (Self::Spec, Self::Factory) {
                (self.spec, self.factory)
            }
        }
    };
}

pub type BoxActionPlugin2 = BoxPlugin<beetry_editor_types::NodeSpec, ActionFactory>;
pub type BoxConditionPlugin2 = BoxPlugin<beetry_editor_types::NodeSpec, ConditionFactory>;
pub type BoxControlPlugin2 = BoxPlugin<beetry_editor_types::NodeSpec, ControlFactory>;

impl Named for beetry_editor_types::NodeSpec {
    fn name(&self) -> &str {
        &self.name().0
    }
}

pub type ActionPluginConstructor2 = PluginConstructor<beetry_editor_types::NodeSpec, ActionFactory>;
pub type ConditionPluginConstructor2 =
    PluginConstructor<beetry_editor_types::NodeSpec, ConditionFactory>;
pub type ControlPluginConstructor2 =
    PluginConstructor<beetry_editor_types::NodeSpec, ControlFactory>;

impl ActionPluginConstructor2 {
    pub fn plugins() -> Result<Vec<BoxActionPlugin2>, PluginError> {
        unique_plugins::<Self, <Self as ConstructPlugin>::Spec, <Self as ConstructPlugin>::Factory>(
        )
    }
}

impl ConditionPluginConstructor2 {
    pub fn plugins() -> Result<Vec<BoxConditionPlugin2>, PluginError> {
        unique_plugins::<Self, <Self as ConstructPlugin>::Spec, <Self as ConstructPlugin>::Factory>(
        )
    }
}

impl ControlPluginConstructor2 {
    pub fn plugins() -> Result<Vec<BoxControlPlugin2>, PluginError> {
        unique_plugins::<Self, <Self as ConstructPlugin>::Spec, <Self as ConstructPlugin>::Factory>(
        )
    }
}

inventory::collect! {ActionPluginConstructor2}
inventory::collect! {ConditionPluginConstructor2}
inventory::collect! {ControlPluginConstructor2}

#[cfg(test)]
mod tests {
    use crate::Plugin;

    use super::*;
    use beetry_plugin_types::node::{ActionLeafSchema, NodeName};

    struct TestPluginA {
        spec: ActionSpec,
        factory: ActionFactory,
    }

    impl Plugin for TestPluginA {
        type Spec = ActionSpec;
        type Factory = ActionFactory;

        fn new() -> Self {
            Self {
                spec: ActionSpec::builder()
                    .name(NodeName::new("TestPlugin"))
                    .schema(ActionLeafSchema::default())
                    .build(),
                factory: ActionFactory::new(Box::new(|_| {
                    Err(anyhow::anyhow!("This is a test factory, not functional"))
                })),
            }
        }

        fn spec(&self) -> &Self::Spec {
            &self.spec
        }

        fn factory(&self) -> &Self::Factory {
            &self.factory
        }

        fn into_parts(self: Box<Self>) -> (Self::Spec, Self::Factory) {
            (self.spec, self.factory)
        }
    }

    struct TestPluginB {
        spec: ActionSpec,
        factory: ActionFactory,
    }

    impl Plugin for TestPluginB {
        type Spec = ActionSpec;
        type Factory = ActionFactory;

        fn new() -> Self {
            Self {
                spec: ActionSpec::builder()
                    .name(NodeName::new("TestPlugin"))
                    .schema(ActionLeafSchema::default())
                    .build(),
                factory: ActionFactory::new(Box::new(|_| {
                    Err(anyhow::anyhow!("This is a test factory, not functional"))
                })),
            }
        }

        fn spec(&self) -> &Self::Spec {
            &self.spec
        }

        fn factory(&self) -> &Self::Factory {
            &self.factory
        }

        fn into_parts(self: Box<Self>) -> (Self::Spec, Self::Factory) {
            (self.spec, self.factory)
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

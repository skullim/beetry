use crate::{BoxPlugin, ConstructPlugin, Named, PluginConstructor, PluginError, unique_plugins};
use anyhow::Result;
use beetry_channel::{AnyBoxReceiver, AnyBoxSender};
use beetry_core::{BoxActionBehavior, BoxConditionBehavior, BoxNode, NonEmptyNodes};
use beetry_editor_types::{output::node::Parameters, spec::node::NodeSpec};
use beetry_reconstruction_types::node::{
    ActionReconstructionData, ConditionReconstructionData, ControlReconstructionData,
};
use bon::Builder;
use std::marker::PhantomData;

type BoxActionFactoryFn = Box<dyn Fn(ActionReconstructionData) -> Result<BoxActionBehavior>>;

pub type ActionFactory = Factory<BoxActionFactoryFn, ActionReconstructionData, BoxActionBehavior>;
pub type ConditionFactory =
    Factory<BoxConditionFactoryFn, ConditionReconstructionData, BoxConditionBehavior>;
type BoxConditionFactoryFn =
    Box<dyn Fn(ConditionReconstructionData) -> Result<BoxConditionBehavior>>;

type BoxControlFactoryFn = Box<dyn Fn(ControlReconstructionData) -> Result<BoxNode>>;
pub type ControlFactory = Factory<BoxControlFactoryFn, ControlReconstructionData, BoxNode>;

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
                let spec = $crate::ActionSpec::builder().name($name).schema(beetry_editor_types::schema!{kind = action, receivers = [$($receiver, desc = $desc),* ]}).build();

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

pub type BoxActionPlugin = BoxPlugin<NodeSpec, ActionFactory>;
pub type BoxConditionPlugin = BoxPlugin<NodeSpec, ConditionFactory>;
pub type BoxControlPlugin = BoxPlugin<NodeSpec, ControlFactory>;

impl Named for NodeSpec {
    fn name(&self) -> &str {
        &self.name().0
    }
}

pub type ActionPluginConstructor = PluginConstructor<NodeSpec, ActionFactory>;
pub type ConditionPluginConstructor = PluginConstructor<NodeSpec, ConditionFactory>;
pub type ControlPluginConstructor = PluginConstructor<NodeSpec, ControlFactory>;

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

#[cfg(test)]
mod tests {
    use crate::Plugin;

    use super::*;
    use beetry_editor_types::spec::node::{NodeKind, NodeName, NodeSpec, NodeSpecKey};

    struct TestPluginA {
        spec: NodeSpec,
        factory: ActionFactory,
    }

    impl Plugin for TestPluginA {
        type Spec = NodeSpec;
        type Factory = ActionFactory;

        fn new() -> Self {
            Self {
                spec: NodeSpec::builder()
                    .key(NodeSpecKey::new(
                        NodeName::new("TestPlugin"),
                        NodeKind::action(),
                    ))
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
        spec: NodeSpec,
        factory: ActionFactory,
    }

    impl Plugin for TestPluginB {
        type Spec = NodeSpec;
        type Factory = ActionFactory;

        fn new() -> Self {
            Self {
                spec: NodeSpec::builder()
                    .key(NodeSpecKey::new(
                        NodeName::new("TestPlugin"),
                        NodeKind::action(),
                    ))
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

use std::{collections::HashSet, marker::PhantomData};

use anyhow::Result;
use beetry_serde::{de::parameter::Parameters, ser::node::LeafSpec};
use bon::Builder;

use crate::{BoxPlugin, ConstructPlugin, Named, Plugin};
use beetry_channel::{AnyBoxReceiver, AnyBoxSender};
use beetry_core::{BoxActionBehavior, BoxConditionBehavior};

pub trait ActionPlugin: Plugin<Spec = LeafSpec, Factory = ActionFactory> {}
impl<P> ActionPlugin for P where P: Plugin<Spec = LeafSpec, Factory = ActionFactory> {}

pub trait ConditionPlugin: Plugin<Spec = LeafSpec, Factory = ConditionFactory> {}
impl<P> ConditionPlugin for P where P: Plugin<Spec = LeafSpec, Factory = ConditionFactory> {}

pub type ActionReconstructionData = NodeReconstructionData<LeafMetadata>;
pub type ConditionReconstructionData = ActionReconstructionData;
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

#[derive(Default)]
pub struct ControlMetadata;

pub type ActionFactory = Factory<BoxActionFactoryFn, ActionReconstructionData, BoxActionBehavior>;
type BoxActionFactoryFn = Box<dyn Fn(ActionReconstructionData) -> Result<BoxActionBehavior>>;

pub type ConditionFactory =
    Factory<BoxConditionFactoryFn, ConditionReconstructionData, BoxConditionBehavior>;
type BoxConditionFactoryFn =
    Box<dyn Fn(ConditionReconstructionData) -> Result<BoxConditionBehavior>>;

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

type BoxLeafPlugin<F> = BoxPlugin<LeafSpec, F>;
pub type BoxActionPlugin = BoxLeafPlugin<ActionFactory>;
pub type BoxConditionPlugin = BoxLeafPlugin<ConditionFactory>;

pub struct PluginConstructor<S, F>(pub fn() -> BoxPlugin<S, F>);

impl<S, F> PluginConstructor<S, F>
where
    S: 'static,
    F: 'static,
{
    pub const fn new<P: Plugin<Spec = S, Factory = F> + 'static>() -> Self {
        Self(|| Box::new(P::new()))
    }
}

impl<S, F> ConstructPlugin for PluginConstructor<S, F>
where
    S: Named,
{
    type Factory = F;
    type Spec = S;
    fn construct(&self) -> BoxPlugin<Self::Spec, Self::Factory> {
        (self.0)()
    }
}

impl Named for LeafSpec {
    fn name(&self) -> &str {
        self.name.0.as_str()
    }
}

pub type ConditionPluginConstructor = PluginConstructor<LeafSpec, ConditionFactory>;
pub type ActionPluginConstructor = PluginConstructor<LeafSpec, ActionFactory>;

impl ActionPluginConstructor {
    pub fn plugins() -> Result<Vec<BoxActionPlugin>, PluginError> {
        unique_plugins::<Self, LeafSpec, ActionFactory>()
    }
}

impl ConditionPluginConstructor {
    pub fn plugins() -> Result<Vec<BoxConditionPlugin>, PluginError> {
        unique_plugins::<Self, LeafSpec, ConditionFactory>()
    }
}

inventory::collect! {ConditionPluginConstructor}
inventory::collect! {ActionPluginConstructor}

#[derive(Debug, Clone, thiserror::Error)]
pub enum PluginError {
    #[error("duplicate plugin name: '{0}'. Each plugin must have a unique name.")]
    DuplicateName(String),
}

fn unique_plugins<C, S, F>() -> Result<Vec<BoxPlugin<S, F>>, PluginError>
where
    S: Named,
    C: inventory::Collect + ConstructPlugin<Spec = S, Factory = F>,
{
    let mut seen_names = HashSet::new();

    inventory::iter::<C>().try_fold(Vec::new(), |mut plugins, constructor| {
        let plugin = constructor.construct();
        let spec = plugin.spec();
        let name = spec.name();

        if seen_names.contains(name) {
            return Err(PluginError::DuplicateName(name.into()));
        }
        seen_names.insert(name.to_string());

        plugins.push(plugin);
        Ok(plugins)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use beetry_serde::ser::node::{LeafKind, LeafSchema, NodeName};

    struct TestPluginA;

    impl Plugin for TestPluginA {
        type Spec = LeafSpec;
        type Factory = ActionFactory;

        fn new() -> Self {
            TestPluginA
        }

        fn spec(&self) -> Self::Spec {
            LeafSpec::new(
                NodeName::new("TestPlugin"),
                LeafSchema::builder().kind(LeafKind::Action).build(),
            )
        }

        fn factory(self: Box<Self>) -> Self::Factory {
            ActionFactory::new(Box::new(|_| {
                Err(anyhow::anyhow!("This is a test factory, not functional"))
            }))
        }
    }

    struct TestPluginB;

    impl Plugin for TestPluginB {
        type Spec = LeafSpec;
        type Factory = ActionFactory;

        fn new() -> Self {
            TestPluginB
        }

        fn spec(&self) -> Self::Spec {
            LeafSpec::new(
                NodeName::new("TestPlugin"),
                LeafSchema::builder().kind(LeafKind::Action).build(),
            )
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

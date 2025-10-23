use std::collections::HashSet;

use anyhow::Result;
use beetry_serde::{
    de::parameter::Parameters,
    ser::node::{LeafSpec, NodeName},
};
use bon::Builder;

use crate::Plugin;
use beetry_channel::{AnyBoxReceiver, AnyBoxSender};
use beetry_core::{BoxActionBehavior, BoxConditionBehavior};

pub trait ActionPlugin: Plugin<Spec = LeafSpec, Factory = ActionFactory> {}
impl<P> ActionPlugin for P where P: Plugin<Spec = LeafSpec, Factory = ActionFactory> {}

pub trait ConditionPlugin: Plugin<Spec = LeafSpec, Factory = ConditionFactory> {}
impl<P> ConditionPlugin for P where P: Plugin<Spec = LeafSpec, Factory = ConditionFactory> {}

#[derive(Builder)]
pub struct NodeReconstructionData {
    #[builder(default)]
    pub receivers: Vec<AnyBoxReceiver>,
    #[builder(default)]
    pub senders: Vec<AnyBoxSender>,
    #[builder(default)]
    pub parameters: Parameters,
}

type BoxActionFactoryFn =
    Box<dyn Fn(NodeReconstructionData) -> Result<BoxActionBehavior> + Send + Sync>;

pub struct ActionFactory {
    func: BoxActionFactoryFn,
}

impl ActionFactory {
    pub fn new(func: BoxActionFactoryFn) -> Self {
        Self { func }
    }

    pub fn try_create(&self, data: NodeReconstructionData) -> Result<BoxActionBehavior> {
        (self.func)(data)
    }
}

type BoxLeafPlugin<F> = Box<dyn Plugin<Spec = LeafSpec, Factory = F>>;
pub type BoxActionPlugin = BoxLeafPlugin<ActionFactory>;
pub type BoxConditionPlugin = BoxLeafPlugin<ConditionFactory>;

trait NodePluginConstructor: Sized {
    type Factory;
    fn construct(&self) -> BoxLeafPlugin<Self::Factory>;
}

pub struct ActionPluginConstructor(pub fn() -> BoxActionPlugin);
impl ActionPluginConstructor {
    pub const fn new<P: ActionPlugin + 'static>() -> Self {
        ActionPluginConstructor(|| Box::new(P::new()))
    }

    pub fn construct(&self) -> BoxActionPlugin {
        (self.0)()
    }

    pub fn plugins() -> Result<Vec<BoxActionPlugin>, PluginError> {
        unique_plugins::<ActionPluginConstructor, ActionFactory>()
    }
}

impl NodePluginConstructor for ActionPluginConstructor {
    type Factory = ActionFactory;
    fn construct(&self) -> BoxLeafPlugin<Self::Factory> {
        (self.0)()
    }
}

inventory::collect!(ActionPluginConstructor);

pub struct ConditionFactory {
    func: Box<dyn Fn(NodeReconstructionData) -> Result<BoxConditionBehavior> + Send + Sync>,
}

impl ConditionFactory {
    pub fn new(
        func: Box<dyn Fn(NodeReconstructionData) -> Result<BoxConditionBehavior> + Send + Sync>,
    ) -> Self {
        Self { func }
    }

    pub fn try_create(&self, data: NodeReconstructionData) -> Result<BoxConditionBehavior> {
        (self.func)(data)
    }
}

pub struct ConditionPluginConstructor(pub fn() -> BoxConditionPlugin);

impl NodePluginConstructor for ConditionPluginConstructor {
    type Factory = ConditionFactory;
    fn construct(&self) -> BoxConditionPlugin {
        (self.0)()
    }
}

impl ConditionPluginConstructor {
    pub const fn new<C: ConditionPlugin + 'static>() -> Self {
        ConditionPluginConstructor(|| Box::new(C::new()))
    }

    pub fn plugins() -> Result<Vec<BoxConditionPlugin>, PluginError> {
        unique_plugins::<ConditionPluginConstructor, ConditionFactory>()
    }
}

inventory::collect!(ConditionPluginConstructor);

#[derive(Debug, Clone, thiserror::Error)]
pub enum PluginError {
    #[error("duplicate plugin name: '{0}'. Each plugin must have a unique name.")]
    DuplicateName(NodeName),
}

fn unique_plugins<C, F>() -> Result<Vec<BoxLeafPlugin<F>>, PluginError>
where
    C: inventory::Collect + NodePluginConstructor<Factory = F>,
{
    let mut seen_names = HashSet::new();

    inventory::iter::<C>().try_fold(Vec::new(), |mut plugins, plugin_constructor| {
        let plugin = plugin_constructor.construct();
        let name = plugin.spec().name;

        if seen_names.contains(&name) {
            return Err(PluginError::DuplicateName(name));
        }
        seen_names.insert(name);

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
            Err(PluginError::DuplicateName(name)) if name == NodeName::new("TestPlugin")
        ));
    }
}

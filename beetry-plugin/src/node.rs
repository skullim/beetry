use anyhow::Result;
use beetry_serde::{de::parameter::Parameters, ser::node::LeafSpec};
use bon::Builder;

use beetry_channel::{AnyBoxReceiver, AnyBoxSender};
use beetry_core::{ActionBehavior, ConditionBehavior};

pub trait NodePlugin: Send + Sync {
    type Spec;
    type Factory;

    fn new() -> Self
    where
        Self: Sized;

    fn spec(&self) -> Self::Spec;

    fn factory(self: Box<Self>) -> Self::Factory;
}

pub type ActionNodePlugin = dyn NodePlugin<Spec = LeafSpec, Factory = ActionFactory>;
pub type ConditionNodePlugin = dyn NodePlugin<Spec = LeafSpec, Factory = ConditionFactory>;

#[derive(Builder)]
pub struct NodeReconstructionData {
    #[builder(default)]
    pub receivers: Vec<AnyBoxReceiver>,
    #[builder(default)]
    pub senders: Vec<AnyBoxSender>,
    #[builder(default)]
    pub parameters: Parameters,
}

pub struct ActionFactory {
    func: Box<dyn Fn(NodeReconstructionData) -> Result<Box<dyn ActionBehavior>> + Send + Sync>,
}

impl ActionFactory {
    pub fn new(
        func: Box<dyn Fn(NodeReconstructionData) -> Result<Box<dyn ActionBehavior>> + Send + Sync>,
    ) -> Self {
        Self { func }
    }

    pub fn try_create(&self, data: NodeReconstructionData) -> Result<Box<dyn ActionBehavior>> {
        (self.func)(data)
    }
}

pub struct ActionNodePluginConstructor(pub fn() -> Box<ActionNodePlugin>);
impl ActionNodePluginConstructor {
    pub const fn new<T: NodePlugin<Spec = LeafSpec, Factory = ActionFactory> + 'static>() -> Self {
        ActionNodePluginConstructor(|| Box::new(T::new()))
    }

    pub fn construct(&self) -> Box<ActionNodePlugin> {
        (self.0)()
    }

    pub fn plugins() -> Vec<Box<ActionNodePlugin>> {
        let mut plugins = vec![];
        for plugin_constructor in inventory::iter::<ActionNodePluginConstructor> {
            let plugin = plugin_constructor.construct();
            plugins.push(plugin);
        }
        plugins
    }
}
inventory::collect!(ActionNodePluginConstructor);

pub struct ConditionFactory {
    func: Box<dyn Fn(NodeReconstructionData) -> Result<Box<dyn ConditionBehavior>> + Send + Sync>,
}

impl ConditionFactory {
    pub fn new(
        func: Box<
            dyn Fn(NodeReconstructionData) -> Result<Box<dyn ConditionBehavior>> + Send + Sync,
        >,
    ) -> Self {
        Self { func }
    }

    pub fn try_create(&self, data: NodeReconstructionData) -> Result<Box<dyn ConditionBehavior>> {
        (self.func)(data)
    }
}

pub struct ConditionNodePluginConstructor(pub fn() -> Box<ConditionNodePlugin>);
impl ConditionNodePluginConstructor {
    pub const fn new<T: NodePlugin<Spec = LeafSpec, Factory = ConditionFactory> + 'static>() -> Self
    {
        ConditionNodePluginConstructor(|| Box::new(T::new()))
    }

    pub fn construct(&self) -> Box<ConditionNodePlugin> {
        (self.0)()
    }

    pub fn plugins() -> Vec<Box<ConditionNodePlugin>> {
        let mut plugins = vec![];
        for plugin_constructor in inventory::iter::<ConditionNodePluginConstructor> {
            let plugin = plugin_constructor.construct();
            plugins.push(plugin);
        }
        plugins
    }
}
inventory::collect!(ConditionNodePluginConstructor);

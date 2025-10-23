use std::collections::HashSet;

use anyhow::Result;
use beetry_serde::{
    de::parameter::Parameters,
    ser::node::{LeafSpec, NodeName},
};
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

pub type ActionPlugin = dyn NodePlugin<Spec = LeafSpec, Factory = ActionFactory>;
pub type ConditionPlugin = dyn NodePlugin<Spec = LeafSpec, Factory = ConditionFactory>;

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

trait NodePluginConstructor: Sized {
    type Factory;

    fn construct(&self) -> Box<dyn NodePlugin<Spec = LeafSpec, Factory = Self::Factory>>;
}

pub struct ActionPluginConstructor(pub fn() -> Box<ActionPlugin>);
impl ActionPluginConstructor {
    pub const fn new<T: NodePlugin<Spec = LeafSpec, Factory = ActionFactory> + 'static>() -> Self {
        ActionPluginConstructor(|| Box::new(T::new()))
    }

    pub fn construct(&self) -> Box<ActionPlugin> {
        (self.0)()
    }

    pub fn plugins() -> Result<Vec<Box<ActionPlugin>>, PluginError> {
        unique_plugins::<ActionPluginConstructor, ActionFactory>()
    }
}

impl NodePluginConstructor for ActionPluginConstructor {
    type Factory = ActionFactory;
    fn construct(&self) -> Box<dyn NodePlugin<Spec = LeafSpec, Factory = Self::Factory>> {
        (self.0)()
    }
}

inventory::collect!(ActionPluginConstructor);

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

pub struct ConditionPluginConstructor(pub fn() -> Box<ConditionPlugin>);

impl NodePluginConstructor for ConditionPluginConstructor {
    type Factory = ConditionFactory;
    fn construct(&self) -> Box<dyn NodePlugin<Spec = LeafSpec, Factory = Self::Factory>> {
        (self.0)()
    }
}

impl ConditionPluginConstructor {
    pub const fn new<T: NodePlugin<Spec = LeafSpec, Factory = ConditionFactory> + 'static>() -> Self
    {
        ConditionPluginConstructor(|| Box::new(T::new()))
    }

    pub fn plugins() -> Result<Vec<Box<ConditionPlugin>>, PluginError> {
        unique_plugins::<ConditionPluginConstructor, ConditionFactory>()
    }
}

inventory::collect!(ConditionPluginConstructor);

#[derive(Debug, Clone, thiserror::Error)]
pub enum PluginError {
    #[error("duplicate plugin name: '{0}'. Each plugin must have a unique name.")]
    DuplicateName(NodeName),
}

fn unique_plugins<C, F>()
-> std::result::Result<Vec<Box<dyn NodePlugin<Spec = LeafSpec, Factory = F>>>, PluginError>
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

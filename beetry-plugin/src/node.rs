use std::{collections::BTreeMap, marker::PhantomData};

use anyhow::Result;
use beetry_channel::{AnyBoxReceiver, AnyBoxSender};
use beetry_core::{BoxActionBehavior, BoxConditionBehavior, BoxNode, NonEmptyNodes};
use beetry_editor_types::{
    output::node::{ParameterValue, Parameters},
    spec::node::{NodeSpec, ParamsSpec, PortKey},
};
use bon::Builder;
use serde::Deserialize;

use crate::{BoxPlugin, ConstructPlugin, Named, PluginConstructor, PluginError, unique_plugins};

pub type LeafReconstructionData = NodeReconstructionData<LeafContext>;
pub type ActionReconstructionData = LeafReconstructionData;
pub type ConditionReconstructionData = LeafReconstructionData;
pub type ControlReconstructionData = NodeReconstructionData<ControlContext>;
pub type DecoratorReconstructionData = NodeReconstructionData<DecoratorContext>;

#[derive(Debug, Builder)]
pub struct NodeReconstructionData<C> {
    pub context: C,
    #[builder(default)]
    pub parameters: Parameters,
}

#[derive(Debug, Default, Builder)]
pub struct LeafContext {
    #[builder(default)]
    pub receivers: BTreeMap<PortKey, AnyBoxReceiver>,
    #[builder(default)]
    pub senders: BTreeMap<PortKey, AnyBoxSender>,
}

impl LeafContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take_receiver(&mut self, key: &PortKey) -> Result<AnyBoxReceiver> {
        self.receivers
            .remove(key)
            .ok_or_else(|| anyhow::anyhow!("failed to obtain receiver for port '{}'", key.as_str()))
    }

    pub fn take_sender(&mut self, key: &PortKey) -> Result<AnyBoxSender> {
        self.senders
            .remove(key)
            .ok_or_else(|| anyhow::anyhow!("failed to obtain sender for port '{}'", key.as_str()))
    }
}

pub struct ControlContext {
    pub children: NonEmptyNodes,
}

impl ControlContext {
    pub fn new(children: NonEmptyNodes) -> Self {
        Self { children }
    }
}

pub struct DecoratorContext {
    pub child: BoxNode,
}

impl DecoratorContext {
    pub fn new(child: BoxNode) -> Self {
        Self { child }
    }
}

/// Provides parameter specification for a parameter type.
///
/// Implement this trait for a serializable params struct to describe which
/// fields can be configured in the editor and how they should be validated.
///
/// ```rust, no_run
/// # use beetry_editor_types::spec::node::{
/// #     FieldDefinition, FieldMetadata, FieldTypeSpec, ParamsSpec,
/// # };
/// # use beetry_plugin::ProvideParamSpec;
/// # use mitsein::iter1::IntoIterator1;
/// use serde::Deserialize;
///
/// /// Not strictly required to implement `ProvideParamSpec` but param type
/// /// must be deserializable to be usable with any node registration macro.
/// /// See book chapter on plugin for in-depth explanation.
/// #[derive(Deserialize)]
/// struct RetryParams;
///
/// impl ProvideParamSpec for RetryParams {
///     fn provide() -> ParamsSpec {
///         [(
///             "retry_limit".into(),
///             FieldDefinition {
///                 type_spec: FieldTypeSpec::U64(FieldMetadata::default()),
///                 description: Some(
///                     "Maximum number of retry attempts".into(),
///                 ),
///             },
///         )]
///         .into_iter1()
///         .collect1()
///     }
/// }
/// ```
pub trait ProvideParamSpec {
    fn provide() -> ParamsSpec;
}

pub struct ParamsDeserializer;

impl ParamsDeserializer {
    pub fn deserialize<T>(params: Parameters) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let deserializer = serde_value::ValueDeserializer::<serde_value::DeserializerError>::new(
            serde_value::Value::Map(
                params
                    .into_iter()
                    .map(|(name, value)| {
                        let value = match value {
                            ParameterValue::Bool(b) => serde_value::Value::Bool(b),
                            ParameterValue::U16(u) => serde_value::Value::U16(u),
                            ParameterValue::U64(u) => serde_value::Value::U64(u),
                            ParameterValue::I64(i) => serde_value::Value::I64(i),
                            ParameterValue::F64(f) => serde_value::Value::F64(f),
                            ParameterValue::String(s) => serde_value::Value::String(s),
                        };
                        (serde_value::Value::String(name), value)
                    })
                    .collect::<BTreeMap<_, _>>(),
            ),
        );
        Ok(T::deserialize(deserializer)?)
    }
}

type BoxActionFactoryFn = Box<dyn Fn(ActionReconstructionData) -> Result<BoxActionBehavior>>;

pub type ActionFactory = Factory<BoxActionFactoryFn, ActionReconstructionData, BoxActionBehavior>;
pub type ConditionFactory =
    Factory<BoxConditionFactoryFn, ConditionReconstructionData, BoxConditionBehavior>;
type BoxConditionFactoryFn =
    Box<dyn Fn(ConditionReconstructionData) -> Result<BoxConditionBehavior>>;

type BoxControlFactoryFn = Box<dyn Fn(ControlReconstructionData) -> Result<BoxNode>>;
pub type ControlFactory = Factory<BoxControlFactoryFn, ControlReconstructionData, BoxNode>;
type BoxDecoratorFactoryFn = Box<dyn Fn(DecoratorReconstructionData) -> Result<BoxNode>>;
pub type DecoratorFactory = Factory<BoxDecoratorFactoryFn, DecoratorReconstructionData, BoxNode>;

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
pub type BoxActionPlugin = BoxPlugin<NodeSpec, ActionFactory>;
pub type BoxConditionPlugin = BoxPlugin<NodeSpec, ConditionFactory>;
pub type BoxControlPlugin = BoxPlugin<NodeSpec, ControlFactory>;
pub type BoxDecoratorPlugin = BoxPlugin<NodeSpec, DecoratorFactory>;

impl Named for NodeSpec {
    fn name(&self) -> &str {
        &self.name().0
    }
}

pub type ActionPluginConstructor = PluginConstructor<NodeSpec, ActionFactory>;
pub type ConditionPluginConstructor = PluginConstructor<NodeSpec, ConditionFactory>;
pub type ControlPluginConstructor = PluginConstructor<NodeSpec, ControlFactory>;
pub type DecoratorPluginConstructor = PluginConstructor<NodeSpec, DecoratorFactory>;

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

impl DecoratorPluginConstructor {
    pub fn plugins() -> Result<Vec<BoxDecoratorPlugin>, PluginError> {
        unique_plugins::<Self, <Self as ConstructPlugin>::Spec, <Self as ConstructPlugin>::Factory>(
        )
    }
}

inventory::collect! {ActionPluginConstructor}
inventory::collect! {ConditionPluginConstructor}
inventory::collect! {ControlPluginConstructor}
inventory::collect! {DecoratorPluginConstructor}

#[cfg(test)]
mod tests {
    use beetry_editor_types::spec::node::{NodeKind, NodeName, NodeSpec, NodeSpecKey};

    use super::*;
    use crate::Plugin;

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

    //@todo registering duplicated entry might affect other tests when plugins()
    //@todo method is called.
    //Better to avoid global registration if possible
    inventory::submit! {
        ActionPluginConstructor::new::<TestPluginB>()
    }

    #[test]
    fn duplicate_plugin_name_error() {
        let result = ActionPluginConstructor::plugins();
        assert!(matches!(
            result,
            Err(PluginError::DuplicateName(name)) if name == NodeName::new("TestPlugin").0
        ));
    }
}

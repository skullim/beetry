use crate::Sequence;
use beetry_core::BoxNode;
use beetry_plugin::{
    Plugin,
    node::{ControlFactory, ControlPluginConstructor, ControlReconstructionData},
};
use beetry_serde::ser::node::{ControlSchema, ControlSpec, NodeName};

struct SequencePlugin;
impl Plugin for SequencePlugin {
    type Spec = ControlSpec;
    type Factory = ControlFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn spec(&self) -> ControlSpec {
        ControlSpec::builder()
            .name(NodeName::new("Sequence"))
            .schema(ControlSchema)
            .build()
    }

    fn factory(self: Box<Self>) -> ControlFactory {
        ControlFactory::new(Box::new(|data: ControlReconstructionData| {
            Ok(Box::new(Sequence::new(data.inner.children)) as BoxNode)
        }))
    }
}

struct FallbackPlugin;
impl Plugin for FallbackPlugin {
    type Spec = ControlSpec;
    type Factory = ControlFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn spec(&self) -> ControlSpec {
        ControlSpec::builder()
            .name(NodeName::new("Fallback"))
            .schema(ControlSchema)
            .build()
    }

    fn factory(self: Box<Self>) -> ControlFactory {
        ControlFactory::new(Box::new(|data: ControlReconstructionData| {
            Ok(Box::new(Sequence::new(data.inner.children)) as BoxNode)
        }))
    }
}

struct ParallelPlugin;
impl Plugin for ParallelPlugin {
    type Spec = ControlSpec;
    type Factory = ControlFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn spec(&self) -> ControlSpec {
        ControlSpec::builder()
            .name(NodeName::new("Parallel"))
            .schema(ControlSchema)
            .build()
    }

    fn factory(self: Box<Self>) -> ControlFactory {
        ControlFactory::new(Box::new(|data: ControlReconstructionData| {
            Ok(Box::new(Sequence::new(data.inner.children)) as BoxNode)
        }))
    }
}

beetry_plugin::submit!(ControlPluginConstructor::new::<SequencePlugin>());
beetry_plugin::submit!(ControlPluginConstructor::new::<FallbackPlugin>());
beetry_plugin::submit!(ControlPluginConstructor::new::<ParallelPlugin>());

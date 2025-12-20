use crate::Sequence;
use beetry_core::BoxNode;
use beetry_plugin::Plugin;
use beetry_plugin::node::{ControlFactory, ControlPluginConstructor, ControlReconstructionData};
use beetry_plugin_types::node::{ControlSchema, ControlSpec, NodeName};

struct SequencePlugin {
    spec: ControlSpec,
    factory: ControlFactory,
}
impl Plugin for SequencePlugin {
    type Spec = ControlSpec;
    type Factory = ControlFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            spec: ControlSpec::builder()
                .name(NodeName::new("Sequence"))
                .schema(ControlSchema)
                .build(),
            factory: ControlFactory::new(Box::new(|data: ControlReconstructionData| {
                Ok(Box::new(Sequence::new(data.inner.children)) as BoxNode)
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

struct FallbackPlugin {
    spec: ControlSpec,
    factory: ControlFactory,
}
impl Plugin for FallbackPlugin {
    type Spec = ControlSpec;
    type Factory = ControlFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            spec: ControlSpec::builder()
                .name(NodeName::new("Fallback"))
                .schema(ControlSchema)
                .build(),
            factory: ControlFactory::new(Box::new(|data: ControlReconstructionData| {
                Ok(Box::new(Sequence::new(data.inner.children)) as BoxNode)
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

struct ParallelPlugin {
    spec: ControlSpec,
    factory: ControlFactory,
}
impl Plugin for ParallelPlugin {
    type Spec = ControlSpec;
    type Factory = ControlFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            spec: ControlSpec::builder()
                .name(NodeName::new("Parallel"))
                .schema(ControlSchema)
                .build(),
            factory: ControlFactory::new(Box::new(|data: ControlReconstructionData| {
                Ok(Box::new(Sequence::new(data.inner.children)) as BoxNode)
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

beetry_plugin::submit!(ControlPluginConstructor::new::<SequencePlugin>());
beetry_plugin::submit!(ControlPluginConstructor::new::<FallbackPlugin>());
beetry_plugin::submit!(ControlPluginConstructor::new::<ParallelPlugin>());

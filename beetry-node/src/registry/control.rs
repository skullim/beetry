use crate::Sequence;
use beetry_core::BoxNode;
use beetry_editor_types::{NodeSpec, NodeSpecKey};
use beetry_plugin::Plugin;
use beetry_plugin::node::{
    ControlFactory, ControlPluginConstructor, ControlPluginConstructor2, ControlReconstructionData,
};
use beetry_plugin_types::node::{ControlSchema, ControlSpec, NodeName};

struct SequencePlugin {
    spec: NodeSpec,
    factory: ControlFactory,
}
impl Plugin for SequencePlugin {
    type Spec = NodeSpec;
    type Factory = ControlFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            spec: NodeSpec::builder()
                .key(NodeSpecKey::new(
                    NodeName::new("Sequence"),
                    beetry_editor_types::NodeKind::Control,
                ))
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

beetry_plugin::submit!(ControlPluginConstructor2::new::<SequencePlugin>());
// beetry_plugin::submit!(ControlPluginConstructor2::new::<FallbackPlugin>());
// beetry_plugin::submit!(ControlPluginConstructor2::new::<ParallelPlugin>());

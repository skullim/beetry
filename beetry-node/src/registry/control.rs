use crate::Sequence;
use beetry_core::BoxNode;
use beetry_editor_types::spec::node::{NodeKind, NodeName, NodeSpec, NodeSpecKey};
use beetry_plugin::Plugin;
use beetry_plugin::node::{ControlFactory, ControlPluginConstructor};
use beetry_reconstruction_types::node::ControlReconstructionData;

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
                    NodeKind::Control,
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
    spec: NodeSpec,
    factory: ControlFactory,
}
impl Plugin for FallbackPlugin {
    type Spec = NodeSpec;
    type Factory = ControlFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            spec: NodeSpec::builder()
                .key(NodeSpecKey::new(
                    NodeName::new("Fallback"),
                    NodeKind::Control,
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

struct ParallelPlugin {
    spec: NodeSpec,
    factory: ControlFactory,
}
impl Plugin for ParallelPlugin {
    type Spec = NodeSpec;
    type Factory = ControlFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            spec: NodeSpec::builder()
                .key(NodeSpecKey::new(
                    NodeName::new("Parallel"),
                    NodeKind::Control,
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

beetry_plugin::submit!(ControlPluginConstructor::new::<SequencePlugin>());
beetry_plugin::submit!(ControlPluginConstructor::new::<FallbackPlugin>());
beetry_plugin::submit!(ControlPluginConstructor::new::<ParallelPlugin>());

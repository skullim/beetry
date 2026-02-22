use crate::{Fallback, Parallel, Sequence};
use beetry_core::BoxNode;
use beetry_editor_types::spec::node::{NodeKind, NodeName, NodeSpec, NodeSpecKey};
use beetry_plugin::node::{ControlFactory, ControlPluginConstructor};
use beetry_plugin::{Plugin, control};
use beetry_reconstruction_types::node::ControlReconstructionData;

control!(
    SequencePlugin: "Sequence";
    children(children),
    create: Sequence::new(children),
);

control!(
    FallbackPlugin: "Fallback";
    children(children),
    create: Fallback::new(children),
);

control!(
    ParallelPlugin: "Parallel";
    children(children),
    create: Parallel::new(children),
);

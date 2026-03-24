use beetry_core::BoxNode;
use beetry_editor_types::spec::node::{NodeKind, NodeName, NodeSpec, NodeSpecKey};
use beetry_plugin::{
    Plugin, control,
    node::{ControlFactory, ControlPluginConstructor, ControlReconstructionData},
};

use crate::{Fallback, MemorySequence, Parallel, ParallelParams, Sequence};

control!(
    SequencePlugin: "Sequence";
    children(children),
    create: Sequence::new(children),
);

control!(
    MemorySequencePlugin: "MemorySequence";
    children(children),
    create: MemorySequence::new(children),
);

control!(
    FallbackPlugin: "Fallback";
    children(children),
    create: Fallback::new(children),
);

control!(
    ParallelPlugin: "Parallel";
    children(children),
    params(parameters): ParallelParams,
    create: Parallel::new(children, parameters),
);

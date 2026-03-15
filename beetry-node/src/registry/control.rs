use beetry_core::BoxNode;
use beetry_editor_types::spec::node::{NodeKind, NodeName, NodeSpec, NodeSpecKey};
use beetry_plugin::{
    Plugin, ProvideParamSpec, control,
    node::{ControlFactory, ControlPluginConstructor, ControlReconstructionData},
};

use crate::{
    Fallback, MemSequence, Parallel, ParallelThreshold, ParallelThresholdParams, Sequence,
};

control!(
    SequencePlugin: "Sequence";
    children(children),
    create: Sequence::new(children),
);

control!(
    MemSequencePlugin: "MemSequence";
    children(children),
    create: MemSequence::new(children),
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

control!(
    ParallelThresholdPlugin: "ParallelThreshold";
    children(children),
    params(parameters): ParallelThresholdParams::provide(),
    create: {
        let params = ParallelThresholdParams::reconstruct(parameters)?;
        ParallelThreshold::new(children, params)
    },
);

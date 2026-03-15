use beetry_core::BoxNode;
use beetry_editor_types::spec::node::{NodeKind, NodeName, NodeSpec, NodeSpecKey};
use beetry_plugin::{
    Plugin, ProvideParamSpec, control,
    node::{ControlFactory, ControlPluginConstructor, ControlReconstructionData},
};

use crate::{Fallback, MemSequence, Parallel, ParallelParams, Sequence};

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
    params(parameters): ParallelParams::provide(),
    create: {
        let params = ParallelParams::reconstruct(parameters)?;
        Parallel::new(children, params)
    },
);

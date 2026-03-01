use crate::{Fail, Invert, Succeed, UntilFailure, UntilSuccess};
use beetry_core::BoxNode;
use beetry_editor_types::spec::node::{NodeKind, NodeName, NodeSpec, NodeSpecKey};
use beetry_plugin::node::{DecoratorFactory, DecoratorPluginConstructor};
use beetry_plugin::{Plugin, decorator};
use beetry_reconstruction_types::node::DecoratorReconstructionData;

decorator!(
    InvertPlugin: "Invert";
    child(child),
    create: Invert::new(child),
);

decorator!(
    SucceedPlugin: "Succeed";
    child(child),
    create: Succeed::new(child),
);

decorator!(
    FailPlugin: "Fail";
    child(child),
    create: Fail::new(child),
);

decorator!(
    UntilSuccessPlugin: "UntilSuccess";
    child(child),
    create: UntilSuccess::new(child),
);

decorator!(
    UntilFailurePlugin: "UntilFailure";
    child(child),
    create: UntilFailure::new(child),
);

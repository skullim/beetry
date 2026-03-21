# Nodes

With communication and parameters in place, we can now describe how nodes are
exposed to the framework.

Beetry provides separate registration macros for each node kind:

- `action!`
- `condition!`
- `control!`
- `decorator!`

Each macro defines the node name, publishes its metadata, and
registers a factory that reconstructs the runtime node from its stored
configuration.

## Basic examples

The following examples show the basic registration form for each node kind.

```rust
use beetry::plugin::action;

action! {
    DetectFlowerPlugin: "DetectFlower";
    create: DetectFlower;
}
```

```rust
use beetry::plugin::condition;

condition! {
    HasTargetPlugin: "HasTarget";
    create: HasTarget;
}
```

```rust
use beetry::plugin::control;

control! {
    SequencePlugin: "Sequence";
    children(children),
    create: Sequence::new(children),
}
```

```rust
use beetry::plugin::decorator;

decorator! {
    InvertPlugin: "Invert";
    child(child),
    create: Invert::new(child),
}
```

## Node With Parameters

When a node needs additional construction-time input, its registration can
publish a parameter specification and reconstruct the typed value before
creating the node.

```rust
use beetry::plugin::{ParamsDeserializer, action};

action! {
    RetryActionPlugin: "RetryAction";
    params(parameters): RetryActionParams::provide();
    create: RetryAction::new(ParamsDeserializer::deserialize(parameters)?);
}
```

In this form, `RetryActionParams::provide()` describes the parameter schema, and
`ParamsDeserializer::deserialize(parameters)?` rebuilds the typed parameter
value for the node constructor.

## Node With Channels

Leaf nodes can also declare typed communication ports during registration. This
makes their inputs and outputs part of the node API.

```rust
use beetry::plugin::action;

action! {
    FollowTrajectoryPlugin: "FollowTrajectory";
    receivers: [trajectory: Trajectory => "Trajectory input"];
    senders: [command: MotionCommand => "Motion command output"];
    create: FollowTrajectory::new(trajectory, command);
}
```

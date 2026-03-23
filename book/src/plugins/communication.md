# Inter-Node Communication

Beetry uses explicit inter-node communication. For a comparison with other
approaches and the rationale behind this decision, see
[Communication Methods Comparison](../appendix/inter-node-communication.md).

## Model

In Beetry nodes communicate through an
opt-in system of typed messages and channels. A node declares its inputs and
outputs, which become the API contract of the node.

This has the benefits:

- messages remain type-safe, with no need for type erasure
- data flow is explicit, which encourages more deliberate design
- dependencies are clear
- the editor can validate more of the tree structure before runtime

Communication in Beetry has two parts:

- messages, which define the typed values exchanged between nodes
- channels, which define how those values are delivered

This communication model is recommended, but not mandatory. Pub/sub frameworks
can still be used internally when they better fit a particular use case,
especially when widely shared data would otherwise require many explicit
communication paths.

## Messages

In Beetry, messages describe the data itself, not the way that data is
transported. For example, a pose estimate, a trajectory, or a safety status can
all be modeled as message types.

To be used as a Beetry message, a type must implement the `Message`
trait. In most cases this is done through the derive macro.

Recommended message types are:

- domain-oriented
- easy to understand without node-specific context
- reusable across multiple nodes

Once a message type exists, it can be exposed to the plugin system and paired
with one of the supported channel kinds. In practice this usually means:

1. define a domain type and derive `Message` for it
2. register it in the plugin system with the `channel!` macro

For example:

```rust
# extern crate beetry;
use beetry::{
    Message,
    type_hash::{self, TypeHash},
};

#[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
}

beetry::plugin::channel! {PoseChannel: Pose}
```

Here, `PoseChannel` is the plugin type, and `Pose` is the message type the
channel will carry.

## Channels

Channels define how messages are delivered between nodes.

In Beetry, channels are built on top of `Sender` and `Receiver`
traits. These traits define the minimal non-blocking interface needed for
communication between nodes.

This makes communication explicit in two ways:

- the message type describes the data contract
- the channel kind describes the delivery behavior

Providing multiple channel implementations matters because different kinds of
data have different runtime needs. Some values should be queued, some should
represent the latest known state, and some should be fanned out to multiple
consumers.

See the [channel library](./channel-library.md) chapter for more details about
the available channel types.

## Data flow

Explicit communication is not only about what data flows through the tree, but
also about when that flow is allowed to happen.

In Beetry, messages are propagated during ticks. This keeps communication
synchronized with node execution, reset, and abort handling. If nodes could
observe or publish arbitrary updates outside the tick-driven execution model,
their internal state could become inconsistent with the tree lifecycle. In
practice, that would make it much harder to reason about whether a value was
produced before an abort or after a reset.

To avoid those issues, Beetry synchronizes message propagation with ticking. At
the same time, users can decide under which node statuses a particular message
is forwarded, preserving full flexibility. In the case of `Action` nodes, this
data flow is typically implemented through action
[hooks](../runtime/execution.md#hooks).

This can introduce some boilerplate; see
[Caveats](../caveats.md#message-propagation) for more details.

# Inter-Node Communication

Most nodes define inputs they consume and outputs they produce. In a behavior
tree, that means data often has to move from one node to another. To keep nodes
modular, reusable, and easy to compose, a behavior tree system needs a clear way
to communicate that data.

One of Beetry's central design goals is explicit communication between
nodes.

## Why explicit communication

Many behavior tree systems rely on a shared blackboard or other globally
accessible state as the default coordination mechanism. That approach can be
flexible, but it also makes dependencies harder to see and can couple nodes to a
particular way of accessing data. A node may read a value that was written
somewhere else in the tree, yet the connection between the two is not obvious
from the tree structure itself. This can make nodes harder to reuse later,
because their data access pattern is embedded in the implementation instead of
being expressed as an explicit interface.

In more complex trees, this becomes even harder to reason about. It is not only
necessary to know that some value exists in shared state, but also to know which
preceding node is responsible for writing it and whether that write is
guaranteed to happen on the relevant execution path. To verify that, a user has
to inspect the implementation details of multiple nodes instead of understanding
the data flow directly from the tree view. That makes the design and change more
error-prone.

## Communication model

Beetry takes a different approach. Nodes communicate through an opt-in system of
typed messages and channels. A node declares its inputs and outputs,
which become the API contract of the node.

This has a few practical benefits:

- messages remain type-safe, with no need for type erasure
- data flow is explicit, which encourages more deliberate design
- dependencies are clear
- the editor can validate more of the tree structure before runtime

Communication in Beetry has two parts:

- messages, which define the typed values exchanged between nodes
- channels, which define how those values are delivered

This communication model is recommended, but not mandatory. Shared state can
still be passed to leaf nodes when needed.

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
is forwarded, preserving full flexibility.

The following sections introduce the mechanisms used to implement explicit
inter-node communication.

## Messages

In Beetry, messages describe the data itself, not the way that data is
transported. For example, a pose estimate, a trajectory, or a safety status can
all be modeled as message types. Channels then define how values of those
types are delivered.

This separation matters because the same kind of data may be useful in
different communication contexts. A message answers the question "what is being
sent?", while a channel answers "how is it delivered?".

In practice, message types are usually simple domain data models, for example:

```rust
use beetry::Message;
use beetry_message::Message;
use type_hash::TypeHash;

#[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
}
```

To be used as a Beetry message, a type must implement the `Message`
trait. In most cases this is done through the derive macro. Message types also
carry stable type metadata used by the framework for registration, typing, and
editor integration.

Recommended message types are:

- domain-oriented
- easy to understand without node-specific context
- reusable across multiple nodes

Once a message type exists, it can be exposed to the plugin system and paired
with one of the supported channel kinds.

## Channels

Channels define how messages are delivered between nodes.

In Beetry, channels are built on top of the core `Sender` and `Receiver`
traits. These traits define the minimal non-blocking interface needed for
communication between nodes.

This makes communication explicit in two ways:

- the message type describes the data contract
- the channel kind describes the delivery behavior

Choosing the right channel matters because different kinds of data have
different runtime needs. Some values should be queued, some should represent
the latest known state, and some should be fanned out to multiple consumers.

Beetry currently provides several channel kinds:

- `mpsc`
- `watch`
- `broadcast`

## Selecting a channel

- use `mpsc` when messages should be processed one by one and every queued item
  matters
- use `watch` when only the latest value matters, such as status or sensor
  state
- use `broadcast` when the same message should be delivered to multiple
  receivers

With `mpsc`, keep in mind that once a sender successfully sends a message,
aborting any intermediate node on the execution path might not clear the
receiver buffer. In the worst case, stale messages can fill the bounded buffer
and cause later sends to fail.
To minimize this risk, the sender and receiver should ideally be neighboring
nodes in the execution path.

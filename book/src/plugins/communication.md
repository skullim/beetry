# Inter-Node Communication

First, we look at the possible communication methods and, based on their
tradeoffs and the goals of the framework, choose the one that best matches
Beetry's internal design goals.

## Methods comparison

### Blackboard

The blackboard is one of the most commonly used data-sharing mechanisms in
behavior tree frameworks. It provides a shared key–value store that allows nodes
to exchange data with minimal infrastructure, making it both flexible and easy
to implement.

However, this flexibility comes at a cost. Data dependencies between nodes
become implicit: it is not clear from the tree definition which node produces a
value or whether it is available when needed. In practice, ensuring correctness
often requires analyzing the entire tree as well as individual node
implementations.

Another issue is that nodes become coupled through shared keys rather than
explicit interfaces. This makes refactoring more error-prone and reduces
modularity. Since values persist in the blackboard, nodes may also read outdated
data, introducing temporal coupling. Additionally, when multiple nodes write to
the same entry, ownership is unclear and later writes may silently overwrite
earlier ones.

BehaviorTree.CPP mitigates some of these issues by introducing input and output
ports, which make data dependencies more explicit and improve readability.
However, ports are still backed by the blackboard, and certain issues remain. In
particular, while ports are typed at the node interface, some type mismatches
may only be detected at runtime when values are accessed.

Finally, in trees that allow parallel execution, for example through a
`Parallel` node, synchronizing concurrent access can be difficult and may impose
an additional runtime penalty.

### Pub/Sub

Like a blackboard, a pub/sub framework allows nodes to exchange data through
shared infrastructure instead of direct references. Unlike a blackboard,
however, pub/sub is message-oriented rather than state-oriented: publishers emit
values to topics or channels, and subscribers receive them asynchronously,
instead of reading and writing persistent entries in a shared store.

However, pub/sub frameworks often introduce framework-level coupling. For nodes
to communicate with each other, they usually need to use the same concrete
pub/sub implementation, or compatible client objects derived from it. As a
result, communication is decoupled at the node level, but tied to a specific
messaging framework at the system level.

Another drawback is a runtime cost. Many pub/sub frameworks are built on IPC or
networking mechanisms, which means messages often have to be serialized and
deserialized before delivery.

### Explicit

In an explicit communication model, data flow is expressed through defined
producer-consumer interfaces. Dependencies
are visible by design, and communication becomes part of the component API
instead of an implementation detail.

This improves modularity and makes API boundaries clearer. It is easier to see
which node produces a value, which node consumes it, and how data moves through
the system. Because communication contracts are explicit, refactoring is safer
and more of the system can be validated before runtime.

However, this clarity comes at the cost of additional structure. Explicit
communication requires more upfront design and more supporting infrastructure.
When the same data has to reach many nodes, the number of communication paths
can also make interfaces more verbose.

Another tradeoff is that communication paths become part of the API. Adding or
removing them may require changes in the affected nodes. On the other hand, this
also makes such changes explicit and allows the compiler to enforce consistency
between producers and consumers.

## Communication model

Beetry selects the explicit communication model. Nodes communicate through an
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

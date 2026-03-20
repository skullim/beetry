# Node Communication

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
node-to-node communication.

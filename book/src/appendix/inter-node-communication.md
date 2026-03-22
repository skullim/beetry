# Inter-Node Communication

## Comparison

This section compares the main approaches to inter-node communication in
behavior trees.

### Blackboard

The blackboard is one of the most commonly used data-sharing mechanisms in
behavior tree frameworks. It provides a shared key-value store that allows
nodes to exchange data with minimal infrastructure, making it both flexible and
easy to implement.

However, this flexibility comes at a cost. Data dependencies between nodes
become implicit: it is not clear from the tree definition which node produces a
value or whether it is available when needed. In practice, ensuring correctness
often requires analyzing the entire tree as well as individual node
implementations.

Another issue is that nodes become coupled through shared keys rather than
explicit interfaces. This makes refactoring more error-prone and reduces
modularity. Since values persist in the blackboard, nodes may also read
outdated data, introducing temporal coupling. Additionally, when multiple nodes
write to the same entry, ownership is unclear and later writes may silently
overwrite earlier ones.

BehaviorTree.CPP mitigates some of these issues by introducing input and output
ports, which make data dependencies more explicit and improve readability.
However, ports are still backed by the blackboard, and certain issues remain.
In particular, while ports are typed at the node interface, some type
mismatches may only be detected at runtime when values are accessed.

Finally, in trees that allow parallel execution, for example through a
`Parallel` node, synchronizing concurrent access can be difficult and may
impose an additional runtime penalty.

### Pub/Sub

Like a blackboard, a pub/sub framework allows nodes to exchange data through
shared infrastructure instead of direct references. Unlike a blackboard,
however, pub/sub is message-oriented rather than state-oriented: publishers
emit values to topics or channels, and subscribers receive them asynchronously,
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
producer-consumer interfaces. Dependencies are visible by design, and
communication becomes part of the component API instead of an implementation
detail.

This improves modularity and makes API boundaries clearer. It is easier to see
which node produces a value, which node consumes it, and how data moves through
the system. Because communication contracts are explicit, refactoring is safer
and more of the system can be validated before runtime.

However, this clarity comes at the cost of additional structure. Explicit
communication requires more upfront design and more supporting infrastructure.
When the same data has to reach many nodes, the number of communication paths
can also make interfaces more verbose.

Another tradeoff is that communication paths become part of the API. Adding or
removing them may require changes in the affected nodes. On the other hand,
this also makes such changes explicit and allows the compiler to enforce
consistency between producers and consumers.

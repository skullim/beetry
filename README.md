# Beetry

<!-- markdownlint-disable MD033 -->
<p align="center">
    <img src="./logo.png" alt="Logo" width="400" height="261">
</p>

## Introduction

Beetry is a framework built around the concept of behavior trees, providing
tools to model, execute, and interact with them.

### What is a behavior tree?

Behavior trees are used to model decision-making and task execution in complex,
reactive systems. They promote composability by breaking behavior into small,
modular nodes that are coordinated by different types of control nodes to model
complex behaviors for different use cases.

They are an alternative to
[finite state machines](https://en.wikipedia.org/wiki/Finite-state_machine),
helping mitigate state explosion and the need to model transitions explicitly.

To read more about behavior trees, see
[behavior tree](https://en.wikipedia.org/wiki/Behavior_tree_(artificial_intelligence,_robotics_and_control)).

## Motivation

The main motivation behind Beetry was to provide a behavior tree framework for
Rust with the following design goals:

- Editor support for designing and reasoning about complex trees
- A scalable and extensible node model, enabling modular, reusable nodes across
  trees
- Explicit data flow between nodes, allowing the tree to be understood directly
  from the editor without inspecting implementation details

There are already quite a few behavior tree libraries in Rust. However, at the
time of writing, none of them fulfilled at least two of these three
requirements. In particular, no Rust library with an integrated editor was
found.

## Comparison with other libraries

The comparison includes [bonsai-bt](https://github.com/Sollimann/bonsai), currently the most popular behavior tree crate in Rust, and [BehaviorTree.CPP](https://github.com/BehaviorTree/BehaviorTree.CPP), one of the most mature open-source behavior tree implementations available.

| Feature | `beetry` | [`bonsai-bt`](https://github.com/Sollimann/bonsai) | [`BehaviorTree.CPP`](https://github.com/BehaviorTree/BehaviorTree.CPP) |
| --- | --- | --- | --- |
| Ecosystem | Rust | Rust | C++ |
| Maturity | Early-stage | Established | Mature |
| Extensibility | Plugin-based custom nodes and channels | Action logic dispatched through enum variants in a central `tick` callback | Custom nodes and runtime plugins |
| Leaf node reusability | ✅ | Limited: leaf nodes are less reusable across trees | ✅ |
| Communication | Explicit inter-node communication through channels | Blackboard-based shared state | Ports and blackboard-based data flow |
| Pub/Sub integration | Any Pub/Sub framework can be used inside custom actions | Not documented | Native ROS2 integration available |
| Ticking policy | User-defined tick source | User calls tree tick | User calls tree tick |
| Action execution model | Action work is executed off the tick thread via registered tasks | Action logic runs in the tree-driven callback model | Tree execution engine is single-threaded; async actions are polled on the tick thread |
| Parallel execution | ✅ Action nodes can be scheduled in parallel by the Parallel control node | ✅ Parallel combinators such as `WhenAll` and `WhenAny` | ❌ |
| Serialization format | JSON | Serde, RON, and Graphviz | XML |

## Editor comparison

Beetry uses an editor called [Beehive](#beehive). At the time of writing,
[`bonsai-bt`](https://github.com/Sollimann/bonsai) does not provide an editor,
while [`BehaviorTree.CPP`](https://github.com/BehaviorTree/BehaviorTree.CPP)
uses a separate editor called [`Groot2`](https://www.behaviortree.dev/groot/).

| Feature | Beehive | [`Groot2`](https://www.behaviortree.dev/groot/) |
| --- | --- | --- |
| Node parameter validation | Early validation during tree design | No documented editor-time validation of arbitrary parameter values |
| Data port validation | Connections are allowed only between matching message types | Connections are based on shared blackboard identifiers; type mismatches may still surface when the tree is loaded or run |
| Tree validation at export | Full: checks tree structure, node connections, and data type compatibility before export | Partial: validates tree structure and node connections, but data type issues may still surface later |
| Dynamic plugins | ❌ No dynamic plugins; plugins are registered at compile time | ✅ Loads custom node models from external files |
| Monitoring and log visualization | ❌ | ✅ (paid above 20 nodes) |
| Breakpoints and fault injection | ❌ | ✅ (paid) |
| Search nodes in tree | ❌ | ✅ (paid) |

## Getting Started

Beetry's public API is provided through the `beetry` crate (see
[API docs](beetry/src/lib.rs)).

If you want to contribute reusable generic nodes or channel implementations,
add them to `beetry-node` and `beetry-channel`, respectively.

### Example

See the [example](beetry-example) crate for an end-to-end autonomous parking
example built on Beetry's plugin system. It shows how domain-specific messages,
channels, and nodes can be exposed to Beehive and executed by the runtime.

## Beehive

Beetry includes Beehive, a visual editor for creating and saving projects, as
well as generating trees to be executed at runtime.

<video controls src="https://github.com/user-attachments/assets/1640300e-9836-462a-af67-8cdbdab43064"></video>

## Roadmap

- [ ] Support external ports
- [ ] Parameters 2.0: improve ergonomics and reusability
- [ ] Subtree support in Beehive
- [ ] User-defined executors
- [ ] Unique compile-time plugin registration
- [ ] Better ergonomics for action node implementation

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>) at your option.

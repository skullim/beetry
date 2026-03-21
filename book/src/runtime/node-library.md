# Node Library

Beetry provides a built-in library of generic nodes in the `beetry-nodes`
crate. When a particular node type is needed, it is a good idea to first check
whether it already exists there before implementing a custom one.

If a generic control node or decorator is missing, consider opening a pull
request to add it to the shared library.

In most cases, implementing a node involves a few steps:

1. Implement the `Node` trait for `YourNodeType`.
2. Register `YourNodeType` as a plugin. See the [Nodes](../plugins/nodes.md)
   chapter for more details.
3. Re-export the node from the `beetry` crate.

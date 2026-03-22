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

For the most up-to-date minimal examples, check the API docs for
`action!`, `condition!`, `control!`, and `decorator!` in the
`beetry::plugin` module.

The registration DSL supports optional parameters for all node kinds. Leaf
nodes can also declare typed communication ports during registration, which
makes their inputs and outputs part of the node API.

>[!TIP]
> If you are adding a generic reusable node, see the
> [Node Library](../runtime/node-library.md) chapter for how to add it to the
> framework itself.

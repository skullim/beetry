# Runtime

At the center of the framework is a simple idea: a behavior tree is built from
nodes arranged in a hierarchy. These nodes are ticked over time according to
that structure, exchange data, and together implement a specific use case.

Beetry keeps modelling, execution, communication, and extensibility separate so
that each part remains clear while the system as a whole stays cohesive.

The framework is built around a few main building blocks:

- modelling concepts, which define the kinds of nodes in a tree and the role
  each of them plays
- execution interfaces, which define how trees are ticked and how nodes return
  their status
- communication interfaces, which define how data moves between nodes
- plugins, which make it possible to extend the framework with custom nodes,
  channels, and other integrations

This is a high-level overview. The following sections each of the listed
building block in more detail.

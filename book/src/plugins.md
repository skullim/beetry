# Plugins

Plugins are the point where an application teaches Beetry about its own tree
vocabulary.

At a high level, a plugin contributes two things:

- a specification, so tooling can discover and validate the extension
- a factory, so runtime values can be created when a saved tree is loaded

The following sections look at the three main extension surfaces: parameters,
nodes, and channels.

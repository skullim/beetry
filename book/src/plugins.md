# Plugins

## Introduction

The plugin system is the mechanism that allows Beetry to be extended with
user-defined types.

## Extension Dimensions

At a high level, a plugin consists of:

- a specification, which tells the framework how to interpret and interact with
  a given plugin type
- a factory, which allows runtime values to be recreated during deserialization

Plugins can extend the framework in two dimensions:

- behavior, by defining a separate plugin type for each node type
- data flow, by defining message types and the channels that transport them

## Registration and Usage

All plugins are registered at compile time. Each plugin must declare a unique
identifier. Registering two plugins of the same type with the same identifier
results in a runtime error. See [Caveats](./caveats.md) for current
limitations.

To make plugins available, the module in which they are defined must be brought
into scope.

## Constraints

As already briefly mentioned in the [Defining Tree](./runtime/defining-tree.md)
chapter plugin-based extensibility constrains the structure.

This introduces the following constraints:

1. Inter-node communication that a node depends on has to be abstracted through
   an interface.
2. Any additional data needed when creating a node has to be expressed in a
   structured way.

The following sections present the concrete solutions to these constraints.

# Beetry

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

## Framework features

- Plugin-based extensibility
- Native asynchronous execution
- Explicit data flow between nodes through channels
- User-defined ticking
- Early validation during tree design

## Getting Started

Beetry's public API is provided through the `beetry` crate (see
[API docs](beetry/src/lib.rs)).

If you want to contribute reusable generic nodes or channel implementations,
add them to `beetry-node` and `beetry-channel`, respectively.

The remaining workspace crates are considered internal APIs and may change
without stability guarantees.

## Editor

Beetry provides an editor for creating and saving projects, as well as
generating trees to be executed at runtime.

<video controls src="https://github.com/user-attachments/assets/1640300e-9836-462a-af67-8cdbdab43064"></video>

## Example

See the [example](beetry-example) crate for an end-to-end autonomous parking
example built on Beetry's plugin system. It shows how domain-specific messages,
channels, and nodes can be exposed to the editor and executed by the runtime.

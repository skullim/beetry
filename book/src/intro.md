# Introduction

Behavior trees are a way to model decision-making and control flow as a
hierarchy of nodes. They are commonly used in games, robotics, and automation to
describe how a system should act, react, and coordinate work. More generally,
they can be applied to problems that fit a tree structure, where non-leaf nodes
control execution and leaf nodes perform work.

Beetry is a behavior tree framework. It combines a runtime for executing trees
with an opt-in GUI editor for creating them along with supporting tooling.

What Beetry offers:

- plugin-based extensibility for domain-specific nodes and messages
- runtime that supports native asynchronous execution
- explicit data flow between nodes through typed channels
- user-defined ticking mechanism
- early validation during tree design
- a native GUI editor

This book introduces the core ideas behind Beetry. It explains how trees are
executed and how nodes communicate, shows how the runtime is used, and covers
plugins and editor integration.

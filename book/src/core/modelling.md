# Modelling Behavior

Behavior trees are structured as hierarchies of nodes. Non-leaf nodes define
how execution flows through the tree, while leaf nodes perform checks or
work.

In Beetry, nodes typically fall into four roles:

- control nodes, which decide how and when their children are ticked
- decorators, which wrap a single child and modify its behavior
- conditions, which evaluate a predicate
- actions, which perform work

Control nodes and decorators shape the structure and control flow of a tree. Conditions and actions sit at the leaves and provide the domain-specific logic for a particular use case.

Leaf behavior is introduced through two main contracts:

- `ConditionBehavior`, which defines the logic evaluated by a `Condition`
- `ActionBehavior`, which defines the logic executed by an `Action`

These contracts let applications inject their own behavior into the tree
without changing the execution model itself. The next section explains how all
of these node kinds participate in the same runtime model through the `Node`
interface.

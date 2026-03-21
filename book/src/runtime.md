# Runtime

Runtime is the part of Beetry that turns a behavior tree from a structural
model into an executable system.

To understand runtime, the first step is to understand how a tree is modelled:
what kinds of nodes exist, how they are arranged in a hierarchy, and what role
each node plays in the overall behavior.

Once that model is clear, the execution model explains how the tree is ticked
over time, how nodes report their status, and how progress is coordinated
through the tree.

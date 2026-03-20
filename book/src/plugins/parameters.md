# Registering Parameters

Parameters describe the configuration a node needs in addition to its position
in the tree and its channel connections. They are the values a user edits in the
editor and that the runtime later uses to reconstruct the node's behavior.

Registering parameters means exposing their schema as part of the node spec.
That schema tells the editor which fields exist, which types they have, and
which values should be collected before a tree can be exported as valid.

This is what keeps the editor and runtime aligned. The same parameter metadata
that drives the configuration UI also lets the reconstruction pipeline
deserialize the stored values back into the typed parameters expected by the
node implementation.

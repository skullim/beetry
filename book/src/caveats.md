# Caveats

## Runtime

### Action abort

Action abort is currently implemented as a blocking operation. Beetry sends an
abort request through the task handle and then polls until the task reports a
terminal status.

In practice this means aborting multiple running nodes is currently handled
sequentially. Beetry sends the abort
request, waits until node reports a terminal status, and
only then continues to the next one.

## Plugins

### Message propagation

When `Action` nodes participate in inter-node communication, data
is produced by the background task and then forwarded via action
hooks. This keeps message propagation synchronized with ticking, but it can
introduce some boilerplate to implement such propagation.

Beetry does not enforce a single propagation policy here, because the right
semantics depend on the use case and on the meaning of the message. This
preserves flexibility, but it also means users must decide when values should be
propagated and when receiver queues should be cleared.

### Duplicate identifiers

Each plugin type must declare a unique identifier. For example it is totally
fine to have control and decorator node with the same name, but two control
nodes with the same name are not supported.

If two plugins of the same type use the same identifier, the conflict is
currently detected at runtime rather than at compile time.

# Caveats

## Runtime

### Action abort

Action abort is currently implemented as a blocking operation. Beetry sends an
abort request through the task handle and then polls until the task reports a
terminal status.

## Plugins

### Duplicate identifiers

Each plugin type must declare a unique identifier. For example it is totally
fine to have control and decorator node with the same name, but two control
nodes with the same name are not supported.

If two plugins of the same type use the same identifier, the conflict is
currently detected at runtime rather than at compile time.

# Channels

Channels define how messages are delivered between nodes.

In Beetry, channels are built on top of the core `Sender` and `Receiver`
traits. These traits define the minimal non-blocking interface needed for
communication between nodes. Leaf nodes consume inputs through receivers and
produce outputs through senders.

This makes communication explicit in two ways:

- the message type describes the data contract
- the channel kind describes the delivery behavior

Choosing the right channel matters because different kinds of data have
different runtime needs. Some values should be queued, some should represent
the latest known state, and some should be fanned out to multiple consumers.

Beetry currently provides several channel kinds through `beetry-channel`:

- `mpsc` for queued point-to-point communication
- `watch` for observing the latest value
- `broadcast` for fan-out communication to multiple receivers

The framework also provides `Input`, a small typed wrapper around a receiver
that is commonly used inside nodes to read incoming values in a structured way.

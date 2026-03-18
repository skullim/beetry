# Channels

Channels define how messages are delivered between nodes.

In Beetry, channels are built on top of the core `Sender` and `Receiver`
traits. These traits define the minimal non-blocking interface needed for
communication between nodes.

This makes communication explicit in two ways:

- the message type describes the data contract
- the channel kind describes the delivery behavior

Choosing the right channel matters because different kinds of data have
different runtime needs. Some values should be queued, some should represent
the latest known state, and some should be fanned out to multiple consumers.

Beetry currently provides several channel kinds:

- `mpsc`
- `watch`
- `broadcast`

## Selecting a channel

- use `mpsc` when messages should be processed one by one and every queued item
  matters
- use `watch` when only the latest value matters, such as status or sensor
  state
- use `broadcast` when the same message should be delivered to multiple
  receivers

With `mpsc`, keep in mind that once a sender successfully sends a message,
aborting any intermediate node on the execution path might not clear the
receiver buffer. In the worst case, stale messages can fill the bounded buffer
and cause later sends to fail.  
To minimize this risk, the sender and receiver should ideally be neighboring nodes in the execution path.
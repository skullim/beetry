# Messages

In Beetry, messages describe the data itself, not the way that data is
transported. For example, a pose estimate, a trajectory, or a safety status can
 all be modeled as message types. Channels then define how values of those
types are delivered.

This separation matters because the same kind of data may be useful in
different communication contexts. A message answers the question "what is being
sent?", while a channel answers "how is it delivered?".

In practice, message types are usually simple domain data models, for example:

```rust
use beetry::Message;
use beetry_message::Message;
use type_hash::TypeHash;

#[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
}
```

To be used as a Beetry message, a type must implement the `Message`
trait. In most cases this is done through the derive macro. Message types also
carry stable type metadata used by the framework for registration, typing, and
editor integration.

Recommended message types are:

- domain-oriented
- easy to understand without node-specific context
- reusable across multiple nodes

Once a message type exists, it can be exposed to the plugin system and paired
with one of the supported channel kinds.

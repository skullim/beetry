# Channels

Unlike the execution model or the communication contracts, the channel library
is not the main abstraction this book needs to explain. Channels are an
implementation detail of how typed messages move between nodes.

Beetry keeps the channel interface in the core communication model, while
concrete reusable channel implementations live separately. Generic channels that
are useful across projects belong in the `beetry-channel` crate. Application-
specific channels are usually introduced through plugins alongside the message
types they carry.

In practice, that means most users do not start from a fixed catalog of
channels. They choose or define the channels that fit their message types and
execution needs, then expose them to the editor and reconstruction pipeline
through the plugin system.

# Beetry

<p align="center">
<img src="./logo.png" alt="My image" width="400" height="261">
</p>

Behavior trees are used to model decision-making and task execution in systems
that must react to changing conditions over time. They are common in robotics,
games, and autonomous systems as they provide a structured way to compose
complex behavior from smaller reusable parts.

Beetry is in an early stage of development. The core ideas are in place, but
the public API may still change between releases.

## Beetry features

- Plugin-based extensibility
- Native asynchronous execution
- Explicit data flow between nodes through channels
- User-defined ticking
- Early validation during tree authoring

For a detailed description of these features, see the `beetry` crate docs in [beetry/src/lib.rs](beetry/src/lib.rs).

## Example

See [beetry-example](beetry-example), which contains an end-to-end autonomous parking example built on Beetry's plugin system. It shows how domain-specific messages, channels, and nodes can be exposed to the editor, serialized into a tree, reconstructed, and executed by the runtime.

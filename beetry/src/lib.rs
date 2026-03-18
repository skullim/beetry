//! # Beetry
//!
//! Beetry is a behavior tree framework that combines an editor for authoring
//! complex trees with a runtime for executing them.
//!
//! The editor is entirely optional: trees can also be built and executed
//! directly in code, and editor support does not affect the runtime model.
//!
//! For the main features and design benefits Beetry offers, see [Why
//! Beetry](#why-beetry).
//!
//! ## Tour of crates
//!
//! The Beetry workspace is split into small crates with focused
//! responsibilities.
//!
//! ### Core runtime
//!
//! - [`beetry_core`]: foundational behavior tree framework traits and concepts.
//! - [`beetry_engine`]: high-level entry point for loading or attaching trees,
//!   starting execution infrastructure, and ticking trees to completion.
//!
//! ### Behavior tree building blocks
//!
//! - [`beetry_exec`]: concrete async executor implementation, including the
//!   framework's default tree executor.
//! - [`beetry_node`]: library of built-in non-leaf behavior tree nodes,
//!   including generic control and decorator nodes.
//! - [`beetry_channel`]: library of supported channel kinds that enables
//!   message passing between leaf nodes.
//!
//! ### Editor
//!
//! - [`beetry_plugin`]: plugin registration and factory system for making
//!   custom nodes and channels available in the editor and reconstructing
//!   authored trees.
//! - [`beetry_editor_types`]: shared types used for editor related crates.
//! - [`beetry_editor_backend`]: backend services for editor operations.
//! - [`beetry_editor_frontend`]: editor frontend implementation.
//!
//! Message types and channel kinds are separate concerns. Registering a
//! message type makes that type available to the framework, while the chosen
//! channel kind determines how values of that type are delivered. The same
//! message type can therefore be used with different channel semantics. See
//! [`beetry_plugin::channel!`] for registering a channel message type and
//! [`beetry_channel`] for the currently supported channel kinds.
//!
//! ## Editor
//!
//! The editor layer lets applications author trees against plugin-provided
//! node and channel specs, then export those authored trees into a validated
//! representation that can be reconstructed at runtime.
//!
//! ### Export pipeline
//!
//! The following diagram shows how plugin-provided specs flow through editor
//! authoring, export validation, and runtime reconstruction.
#![doc = simple_mermaid::mermaid!("../docs/tree_export_chart.mmd")]
//!
//! ## Why Beetry
//!
//! ### Data flow between nodes through channels
//!
//! Many behavior tree frameworks rely on a shared blackboard or other globally
//! accessible state. That approach is flexible, but it often makes trees harder
//! to understand over time. Data dependencies become implicit: a node reads
//! some value from shared state, another node writes it somewhere else, and the
//! connection between the two is not visible in the tree itself. As the system
//! grows, this creates hidden coupling and makes debugging more difficult.
//!
//! Beetry takes a different approach. Nodes communicate through typed channels,
//! which makes data flow explicit and visible. A node depends on specific
//! inputs and produces specific outputs, and those connections are part of the
//! authored tree rather than an informal convention. This makes trees easier to
//! reason about, easier to review, and easier to change safely. It also
//! encourages cleaner system design by reducing reliance on global mutable
//! state and replacing it with message passing between well-defined components.
//!
//! ### Plugin-based extensibility
//!
//! Behavior tree frameworks are most useful when they can adapt to the domain
//! they are used in. In practice, that means the framework needs to support
//! custom node types, custom message types, and domain-specific communication
//! patterns without forcing users to modify the framework internals.
//!
//! Beetry is built around plugin-based registration of nodes and channels. This
//! lets applications extend the system in a structured way while keeping the
//! core runtime independent of domain logic. The result is a framework that
//! stays small and reusable, while still allowing projects to define their own
//! behavior vocabulary. This is especially useful in larger systems, where
//! different subsystems may need their own specialized nodes, message schemas,
//! or integration points.
//!
//! ### Native asynchronous execution
//!
//! In many real applications, behavior tree actions are not instantaneous. They
//! may wait for external responses, perform I/O, run background work, or need
//! to be cancelled when the tree changes direction.
//!
//! Beetry is designed with asynchronous execution in mind. Long-running actions
//! can be represented directly in the execution model instead of being
//! awkwardly simulated through repeated polling and external bookkeeping. This
//! makes the framework a better fit for modern systems that interact with
//! services, devices, sensors, or other concurrent components. It also gives
//! the runtime a clearer model for status tracking, cancellation, and task
//! lifecycle management.
//!
//! ### User-defined ticking
//!
//! Beetry does not hardcode a single ticking strategy. Trees are driven by a
//! `Ticker` built from any `Stream<Item = ()>`, which means applications can
//! decide exactly when ticks happen.
//!
//! For general use, `PeriodicTick` is the default choice. If an application
//! needs something more specific, such as combining a periodic heartbeat with
//! external wake-up signals, it can provide its own stream without changing the
//! behavior tree runtime itself. This makes the ticking model flexible enough
//! for simple loops, event-driven systems, and mixed execution strategies.
//!
//! ### Flexible communication model
//!
//! Different domains need different ways of moving information between nodes.
//! Some nodes need simple one-way signals, while others need richer message
//! flows or integration with external systems. A behavior tree framework
//! becomes much more useful when its communication model can support a range of
//! these use cases without collapsing back into unstructured shared state.
//!
//! Beetry's typed channel system is intended to support a wide range of
//! communication patterns while preserving explicit structure and type safety.
//! This gives users more flexibility in how they model interactions between
//! nodes, without losing the benefits of clear dependencies and well-defined
//! message contracts. That combination is important for scaling from simple
//! examples to larger real-world applications.
//!
//! ### Safe tree authoring
//!
//! Visual editors are useful, but in many tools they are mostly drawing
//! surfaces: they let users assemble a tree, but many important mistakes are
//! only discovered later when the tree is executed. That slows iteration and
//! makes authored trees less trustworthy.
//!
//! Beetry treats the editor as part of the correctness story, not just a
//! convenience layer. Parameter validation happens when nodes are created and
//! configured, so invalid values can be caught early. Invalid tree structures
//! are also detected during export, which helps ensure that a tree leaving the
//! editor is already structurally sound. This shortens the feedback loop,
//! reduces avoidable runtime failures, and makes the editor a more reliable
//! tool for building production trees.

pub mod channel;
#[cfg(feature = "editor")]
pub mod editor;
pub mod leaf;
pub mod node;
#[cfg(feature = "plugin")]
pub mod plugin;
pub mod runtime;

pub use beetry_macros::Message;
pub use beetry_message::Message;

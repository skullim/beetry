#![warn(missing_docs)]

//! `beetry` is the main public API of the framework. APIs exposed by member
//! crates are not considered public API and should not be used directly.
//!
//! For concepts and guides see the Beetry book.
//!
//! ## Feature flags
//!
//! - `plugin`: enables plugin-based registration for custom nodes and channels.
//!   This can be needed when one wants to read and execute a serialized tree
//!   that contains user-defined nodes or channels.
//! - `editor`: enables Beehive for project or tree editing.
//! - `tokio`: enables Tokio-backed channels.
//!
//! None of the features are required to execute a tree defined in code.

#[cfg(feature = "plugin")]
pub mod channel;
#[cfg(feature = "editor")]
pub mod editor;
pub mod leaf;
pub mod node;
#[cfg(feature = "plugin")]
pub mod plugin;
pub mod runtime;

#[cfg(feature = "plugin")]
pub use beetry_macros::Message;
#[cfg(feature = "plugin")]
pub use beetry_message::Message;
#[cfg(feature = "plugin")]
pub use beetry_message::type_hash;

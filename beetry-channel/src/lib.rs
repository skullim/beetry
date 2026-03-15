//! Channel utilities and concrete channel backends for Beetry.
//!
//! This crate builds on the core sender/receiver traits from
//! [`beetry_core`] and provides:
//! - [`Input`], a small typed wrapper around a receiver used by nodes
//! - type-erased [`AnyBoxSender`] and [`AnyBoxReceiver`] helpers
//! - [`external`] support for registering externally provided receivers
//! - optional [`tokio`] channel implementations behind the `tokio` feature
//!
//! Each channel type provides different communication semantics. Currently
//! supported channels are:
//! - `mpsc`
//! - `watch`
//! - `broadcast`

mod any;
pub use any::{AnyBoxReceiver, AnyBoxSender};

pub mod external;
pub mod input;

#[cfg(feature = "tokio")]
pub mod tokio;

// reexport for macro
pub use anyhow;
pub use bon::{bon, builder};
pub use input::Input;
pub use tupleops;

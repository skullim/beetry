//! This crate defines the sender/receiver traits and related types used for
//! plugin registration and explicit inter-node communication.
//!
//! It provides:
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
//!
//! If you want to add a new channel implementation, this is the crate where it
//! should be defined.

mod any;
mod contract;
pub use any::{AnyBoxReceiver, AnyBoxSender};
pub use contract::{BoxReceiver, BoxSender, Receiver, Sender, TryRecvResult, TrySendResult, error};

pub mod external;
pub mod input;

#[cfg(feature = "tokio")]
pub mod tokio;

// reexport for macro
pub use anyhow;
pub use bon::{bon, builder};
pub use input::Input;

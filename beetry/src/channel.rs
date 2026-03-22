//! Typed communication primitives and built-in channel implementations.

#[cfg(feature = "tokio")]
pub use beetry_channel::tokio;
pub use beetry_channel::{AnyBoxReceiver, AnyBoxSender, Input};
pub use beetry_core::{
    BoxReceiver, BoxSender, Receiver, Sender, TryRecvResult, TrySendResult,
    error::{TryRecvError, TrySendError},
};
pub use beetry_macros::receivers;

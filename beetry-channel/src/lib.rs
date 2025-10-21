mod any;
pub use any::{AnyBoxReceiver, AnyBoxSender};

pub mod external;
mod input;

pub use input::{Input, Metadata};
#[cfg(feature = "tokio")]
pub mod tokio;

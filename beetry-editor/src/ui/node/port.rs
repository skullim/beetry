pub mod input;
pub mod output;
pub mod receiver;
pub mod sender;

pub use receiver::{Receiver, Receiver2};
pub use sender::{Sender, Sender2};

use dioxus::prelude::*;

pub(super) fn style_defs() -> Element {
    rsx! {
        {input::style_defs()}
        {output::style_defs()}
    }
}

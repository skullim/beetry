pub mod input;
pub mod output;
pub mod receiver;
pub mod sender;

pub use receiver::Receiver;
pub use sender::Sender;

use dioxus::prelude::*;

pub(super) fn style_defs() -> Element {
    rsx! {
        {input::style_defs()}
        {output::style_defs()}
    }
}

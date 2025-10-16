pub(crate) mod input;
pub(crate) mod output;
pub(crate) mod receiver;
pub(crate) mod sender;

pub(crate) use receiver::Receiver;
pub(crate) use sender::Sender;

use dioxus::prelude::*;

pub(super) fn style_defs() -> Element {
    rsx! {
        {input::style_defs()}
        {output::style_defs()}
    }
}

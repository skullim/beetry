pub mod input;
pub mod menu;
pub mod output;
pub mod receiver;
pub mod sender;

pub use menu::Menu;
pub use receiver::Receiver;
pub use sender::Sender;

use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionOrigin {
    Sender,
    Receiver,
}

pub(super) fn style_defs() -> Element {
    rsx! {
        defs {
            linearGradient { id: "io-port-gradient",
                stop { offset: "0%", stop_color: "#8B5CF6" }
                stop { offset: "100%", stop_color: "#7C3AED" }
            }

            linearGradient { id: "io-port-hover",
                stop { offset: "0%", stop_color: "#A78BFA" }
                stop { offset: "100%", stop_color: "#8B5CF6" }
            }
        }
    }
}

pub(super) struct IoPortStyleUrl;
impl IoPortStyleUrl {
    pub const GRADIENT: &'static str = "url(#io-port-gradient)";
    pub const HOVER: &'static str = "url(#io-port-hover)";
}

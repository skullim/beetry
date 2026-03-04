pub mod input;
pub mod output;

use dioxus::prelude::*;

pub(super) fn style_defs() -> Element {
    rsx! {
        defs {
            linearGradient { id: "io-pin-gradient",
                stop { offset: "0%", stop_color: "#8B5CF6" }
                stop { offset: "100%", stop_color: "#7C3AED" }
            }

            linearGradient { id: "io-pin-hover",
                stop { offset: "0%", stop_color: "#A78BFA" }
                stop { offset: "100%", stop_color: "#8B5CF6" }
            }
        }
    }
}

pub(super) struct IoPinStyleUrl;
impl IoPinStyleUrl {
    pub const GRADIENT: &'static str = "url(#io-pin-gradient)";
    pub const HOVER: &'static str = "url(#io-pin-hover)";
}

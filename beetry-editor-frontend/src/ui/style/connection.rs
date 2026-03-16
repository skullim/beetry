use dioxus::prelude::*;

pub struct Stroke;
impl Stroke {
    // no need for stroke by default
    pub const DEFAULT: &'static str = "none";
    pub const UNCONNECTED_DEFAULT: &'static str = "#b9626d";
    pub const UNCONNECTED_HOVER: &'static str = "#d98892";
    pub const WIDTH: &'static str = "2";
}

pub struct Fill;
impl Fill {
    pub const UNCONNECTED: &'static str = "rgba(0,0,0,0)";
    pub const SENDER: &'static str = "url(#channel-sender-gradient)";
    pub const EXTERNAL_PORT: &'static str = "url(#channel-external-port-gradient)";
    pub const BODY: &'static str = "url(#channel-body-gradient)";
    pub const RECEIVER: &'static str = "url(#channel-receiver-gradient)";
}

pub struct FillHover;
impl FillHover {
    pub const SENDER: &'static str = "url(#channel-sender-gradient-hover)";
    pub const EXTERNAL_PORT: &'static str = "url(#channel-external-port-gradient-hover)";
    pub const BODY: &'static str = "url(#channel-body-gradient-hover)";
    pub const RECEIVER: &'static str = "url(#channel-receiver-gradient-hover)";
}

pub fn style_defs() -> Element {
    rsx! {
        defs {
            linearGradient { id: "channel-sender-gradient",
                stop { offset: "5%", stop_color: "#10B981" }
                stop { offset: "95%", stop_color: "#059669" }
            }

            linearGradient { id: "channel-external-port-gradient",
                stop { offset: "5%", stop_color: "#24272cff" }
                stop { offset: "95%", stop_color: "#0a0c0fff" }
            }

            linearGradient { id: "channel-body-gradient",
                stop { offset: "5%", stop_color: "#3B82F6" }
                stop { offset: "95%", stop_color: "#1D4ED8" }
            }

            linearGradient { id: "channel-receiver-gradient",
                stop { offset: "5%", stop_color: "#6B7280" }
                stop { offset: "95%", stop_color: "#374151" }
            }

            linearGradient { id: "channel-sender-gradient-hover",
                stop { offset: "5%", stop_color: "#34D399" }
                stop { offset: "95%", stop_color: "#10B981" }
            }

            linearGradient { id: "channel-external-port-gradient-hover",
                stop { offset: "5%", stop_color: "#393d44ff" }
                stop { offset: "95%", stop_color: "#14171bff" }
            }

            linearGradient { id: "channel-body-gradient-hover",
                stop { offset: "5%", stop_color: "#60A5FA" }
                stop { offset: "95%", stop_color: "#3B82F6" }
            }

            linearGradient { id: "channel-receiver-gradient-hover",
                stop { offset: "5%", stop_color: "#9CA3AF" }
                stop { offset: "95%", stop_color: "#6B7280" }
            }

        }
    }
}

use dioxus::prelude::*;

pub fn style_defs() -> Element {
    rsx! {
        defs {
            filter { id: "shadow",
                feDropShadow {
                    dx: "2",
                    dy: "3",
                    std_deviation: "3",
                    flood_color: "rgba(0,0,0,0.2)",
                    flood_opacity: "1",
                }
            }

            filter { id: "shadow-hover",
                feDropShadow {
                    dx: "3",
                    dy: "4",
                    std_deviation: "4",
                    flood_color: "rgba(0,0,0,0.3)",
                    flood_opacity: "1",
                }
            }
        }
    }
}

pub struct FilterUrl;

impl FilterUrl {
    pub const SHADOW: &'static str = "url(#shadow)";
    pub const SHADOW_HOVER: &'static str = "url(#shadow-hover)";
}

use dioxus::prelude::*;

#[component]
pub(crate) fn Topbar() -> Element {
    let mut is_visible = use_signal(|| false);

    rsx! {
        div { class: "bt-topbar",
            div {
                class: "bt-topbar-help",
                tabindex: "0",
                onmouseenter: move |_| is_visible.set(true),
                onmouseleave: move |_| is_visible.set(false),
                "?"
            }
            if is_visible() {
                div { class: "bt-topbar-help-tooltip",
                    p {
                        "Zoom: "
                        span { class: "bt-topbar-chip", "Ctrl + Wheel" }
                    }
                    p { "Create node edge: Drag parent -> child port." }
                    p { "Create channel edge: Drag node data port -> matching channel port." }
                }
            }
        }
    }
}

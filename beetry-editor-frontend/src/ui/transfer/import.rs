use dioxus::prelude::*;

use crate::ui::handler::define_handlers;

define_handlers!(on_click: ());

#[component]
pub fn Import() -> Element {
    let handlers = use_context::<Handlers>();
    rsx! {
        button {
            class: "bt-btn bt-btn--toolbar",
            onclick: move |_| { handlers.on_click.call(()) },
            "Import project"
        }
    }
}

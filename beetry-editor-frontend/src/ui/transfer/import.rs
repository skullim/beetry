use dioxus::prelude::*;

use crate::ui::handler::handlers;

handlers!(on_click: ());

#[component]
pub fn Import() -> Element {
    let handlers = use_context::<Handlers>();
    rsx! {
        button { onclick: move |_| { handlers.on_click.call(()) },
            "Import project"
        }
    }
}

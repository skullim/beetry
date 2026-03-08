use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct Props {
    pub query: Signal<String>,
}

#[component]
pub(crate) fn Search(query: Signal<String>) -> Element {
    rsx! {
        input {
            class: "bt-sidebar-search",
            r#type: "search",
            value: "{query()}",
            placeholder: "Search nodes and channels...",
            oninput: move |evt| query.set(evt.value()),
        }
    }
}

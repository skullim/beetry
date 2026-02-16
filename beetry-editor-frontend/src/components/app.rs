mod context;

use dioxus::prelude::*;

use super::editor::Editor;

#[component]
pub(crate) fn App() -> Element {
    rsx! {
        context::Provider {
            Editor {}
        }
    }
}
